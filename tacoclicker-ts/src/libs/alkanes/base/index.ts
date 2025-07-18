import { IOylProvider } from "../provider/types";
import * as bitcoin from "bitcoinjs-lib";
import {
  encipher,
  encodeRunestoneProtostone,
  p2tr_ord_reveal,
  ProtoStone,
} from "alkanes/lib/index";
import { ProtoruneEdict } from "alkanes/lib/protorune/protoruneedict";
import { IAccount } from "../account/types";
import { Signer } from "../signer";
import {
  findXAmountOfSats,
  getOutputValueByVOutIndex,
  getVSize,
  inscriptionSats,
  tweakSigner,
} from "../shared/utils";
import { formatInputsToSign } from "../base/utils";
import { getEstimatedFee } from "../psbt";
import { OylTransactionError } from "../shared/errors";
import { AlkanesPayload } from "../shared/types";
import { getAddressType } from "../account/utils";
import { toXOnly } from "bitcoinjs-lib/src/psbt/bip371";
import { LEAF_VERSION_TAPSCRIPT } from "bitcoinjs-lib/src/payments/bip341";
import { selectSpendableUtxos } from "../utxo/utils";
import { FormattedUtxo } from "../utxo/types";
import { calculateTaprootTxSize } from "../shared/utils";
import { DEFAULT_FEE_RATE } from "../shared/constants";
import { addInputForUtxo } from "../base/utils";
export const actualDeployCommitFee = async ({
  payload,
  tweakedPublicKey,
  utxos,
  account,
  provider,
  feeRate,
}: {
  payload: AlkanesPayload;
  tweakedPublicKey: string;
  utxos: FormattedUtxo[];
  account: IAccount;
  provider: IOylProvider;
  feeRate?: number;
}) => {
  if (!feeRate) {
    feeRate = (await provider.esplora.getFeeEstimates())["1"] as number;
  }

  const { psbt } = await createDeployCommitPsbt({
    payload,
    utxos,
    tweakedPublicKey,
    account,
    provider,
    feeRate,
  });

  const { fee: estimatedFee } = await getEstimatedFee({
    feeRate,
    psbt,
    provider,
  });

  const { psbt: finalPsbt } = await createDeployCommitPsbt({
    payload,
    utxos,
    tweakedPublicKey,
    account,
    provider,
    feeRate,
    fee: estimatedFee,
  });

  const { fee: finalFee, vsize } = await getEstimatedFee({
    feeRate,
    psbt: finalPsbt,
    provider,
  });

  return { fee: finalFee, vsize };
};

export const minimumFee = ({
  taprootInputCount,
  nonTaprootInputCount,
  outputCount,
}: {
  taprootInputCount: number;
  nonTaprootInputCount: number;
  outputCount: number;
}) => {
  return calculateTaprootTxSize(
    taprootInputCount,
    nonTaprootInputCount,
    outputCount
  );
};

export interface ProtostoneMessage {
  protocolTag?: bigint;
  edicts?: ProtoruneEdict[];
  pointer?: number;
  refundPointer?: number;
  calldata: bigint[];
}

export const encodeProtostone = ({
  protocolTag = 1n,
  edicts = [],
  pointer = 0,
  refundPointer = 0,
  calldata,
}: ProtostoneMessage) => {
  return encodeRunestoneProtostone({
    protostones: [
      ProtoStone.message({
        protocolTag,
        edicts,
        pointer,
        refundPointer,
        calldata: encipher(calldata),
      }),
    ],
  }).encodedRunestone;
};

export const createExecutePsbt = async ({
  alkanesUtxos,
  frontendFee,
  feeAddress,
  utxos,
  account,
  protostone,
  provider,
  feeRate,
  fee = 0,
}: {
  alkanesUtxos?: FormattedUtxo[];
  frontendFee?: bigint;
  feeAddress?: string;
  utxos: FormattedUtxo[];
  account: IAccount;
  protostone: Buffer;
  provider: IOylProvider;
  feeRate?: number;
  fee?: number;
}) => {
  try {
    const SAT_PER_VBYTE = feeRate ?? DEFAULT_FEE_RATE;
    const MIN_RELAY = 546n;

    if (frontendFee && !feeAddress) {
      throw new Error("feeAddress required when frontendFee is set");
    }

    const feeSatEffective: bigint =
      frontendFee && frontendFee >= MIN_RELAY ? frontendFee : 0n;

    const spendTargets = 546 + Number(feeSatEffective);

    const minTxSize = minimumFee({
      taprootInputCount: 2,
      nonTaprootInputCount: 0,
      outputCount: 2 + (feeSatEffective > 0n ? 1 : 0),
    });

    const minFee = Math.max(minTxSize * SAT_PER_VBYTE, 250);
    let minerFee = fee === 0 ? minFee : fee;

    let gatheredUtxos = {
      utxos: utxos,
      totalAmount: utxos.reduce((acc, utxo) => acc + utxo.satoshis, 0),
    };

    const satsNeeded = spendTargets + minerFee;
    let foundUtxos = findXAmountOfSats(gatheredUtxos.utxos, satsNeeded);

    if (fee === 0 && foundUtxos.utxos.length > 1) {
      const newSize = minimumFee({
        taprootInputCount: foundUtxos.utxos.length,
        nonTaprootInputCount: 0,
        outputCount: 2 + (feeSatEffective > 0n ? 1 : 0),
      });
      minerFee = Math.max(newSize * SAT_PER_VBYTE, 250);
      if (foundUtxos.totalAmount < minerFee) {
        throw new OylTransactionError(Error("Insufficient balance"));
      }
    }

    const psbt = new bitcoin.Psbt({ network: provider.network });

    if (alkanesUtxos) {
      for (const utxo of alkanesUtxos) {
        await addInputForUtxo(psbt, utxo, account, provider);
      }
    }
    for (const utxo of gatheredUtxos.utxos) {
      await addInputForUtxo(psbt, utxo, account, provider);
    }

    psbt.addOutput({ address: account.taproot.address, value: 546 });
    psbt.addOutput({ script: protostone, value: 0 });

    if (feeSatEffective > 0n) {
      psbt.addOutput({
        address: feeAddress!,
        value: Number(feeSatEffective),
      });
    }

    const totalAlkanesAmount = alkanesUtxos
      ? alkanesUtxos.reduce((acc, utxo) => acc + utxo.satoshis, 0)
      : 0;

    const inputsTotal = gatheredUtxos.totalAmount + (totalAlkanesAmount ?? 0);
    const outputsTotal = psbt.txOutputs.reduce((sum, o) => sum + o.value, 0);

    let change = inputsTotal - outputsTotal - minerFee;
    if (change < 0)
      throw new OylTransactionError(Error("Insufficient balance"));

    if (change >= Number(MIN_RELAY)) {
      psbt.addOutput({
        address: account[account.spendStrategy.changeAddress].address,
        value: change,
      });
    } else {
      minerFee += change;
      change = 0;
    }

    const formatted = await formatInputsToSign({
      _psbt: psbt,
      senderPublicKey: account.taproot.pubkey,
      network: provider.network,
    });

    return {
      psbt: formatted.toBase64(),
      psbtHex: formatted.toHex(),
    };
  } catch (err) {
    throw err;
  }
};

export const createDeployCommitPsbt = async ({
  payload,
  utxos,
  tweakedPublicKey,
  account,
  provider,
  feeRate,
  fee,
}: {
  payload: AlkanesPayload;
  utxos: FormattedUtxo[];
  tweakedPublicKey: string;
  account: IAccount;
  provider: IOylProvider;
  feeRate?: number;
  fee?: number;
}) => {
  try {
    let gatheredUtxos = selectSpendableUtxos(utxos, account.spendStrategy);
    feeRate = feeRate ?? DEFAULT_FEE_RATE;
    const minFee = minimumFee({
      taprootInputCount: 2,
      nonTaprootInputCount: 0,
      outputCount: 2,
    });
    const calculatedFee = minFee * feeRate < 250 ? 250 : minFee * feeRate;
    let finalFee = fee ? fee : calculatedFee;

    let psbt = new bitcoin.Psbt({ network: provider.network });

    const script = Buffer.from(
      p2tr_ord_reveal(toXOnly(Buffer.from(tweakedPublicKey, "hex")), [payload])
        .script
    );

    const inscriberInfo = bitcoin.payments.p2tr({
      internalPubkey: toXOnly(Buffer.from(tweakedPublicKey, "hex")),
      scriptTree: {
        output: script,
      },
      network: provider.network,
    });

    if (!inscriberInfo.address) {
      throw new OylTransactionError(
        new Error("Failed to derive P2TR address for inscriber")
      );
    }

    const wasmDeploySize = getVSize(Buffer.from(payload.body)) * feeRate;

    gatheredUtxos = findXAmountOfSats(
      [...utxos],
      wasmDeploySize + Number(inscriptionSats) + finalFee * 2
    );

    if (!fee && gatheredUtxos.utxos.length > 1) {
      const txSize = minimumFee({
        taprootInputCount: gatheredUtxos.utxos.length,
        nonTaprootInputCount: 0,
        outputCount: 2,
      });
      finalFee = txSize * feeRate < 250 ? 250 : txSize * feeRate;

      if (gatheredUtxos.totalAmount < finalFee) {
        gatheredUtxos = findXAmountOfSats(
          [...utxos],
          wasmDeploySize + Number(inscriptionSats) + finalFee * 2
        );
      }
    }

    for (let i = 0; i < gatheredUtxos.utxos.length; i++) {
      if (getAddressType(gatheredUtxos.utxos[i].address) === 0) {
        const previousTxHex: string = await provider.esplora.getTxHex(
          gatheredUtxos.utxos[i].txId
        );
        psbt.addInput({
          hash: gatheredUtxos.utxos[i].txId,
          index: gatheredUtxos.utxos[i].outputIndex,
          nonWitnessUtxo: Buffer.from(previousTxHex, "hex"),
        });
      }
      if (getAddressType(gatheredUtxos.utxos[i].address) === 2) {
        const redeemScript = bitcoin.script.compile([
          bitcoin.opcodes.OP_0,
          bitcoin.crypto.hash160(
            Buffer.from(account.nestedSegwit.pubkey, "hex")
          ),
        ]);

        psbt.addInput({
          hash: gatheredUtxos.utxos[i].txId,
          index: gatheredUtxos.utxos[i].outputIndex,
          redeemScript: redeemScript,
          witnessUtxo: {
            value: gatheredUtxos.utxos[i].satoshis,
            script: bitcoin.script.compile([
              bitcoin.opcodes.OP_HASH160,
              bitcoin.crypto.hash160(redeemScript),
              bitcoin.opcodes.OP_EQUAL,
            ]),
          },
        });
      }
      if (
        getAddressType(gatheredUtxos.utxos[i].address) === 1 ||
        getAddressType(gatheredUtxos.utxos[i].address) === 3
      ) {
        psbt.addInput({
          hash: gatheredUtxos.utxos[i].txId,
          index: gatheredUtxos.utxos[i].outputIndex,
          witnessUtxo: {
            value: gatheredUtxos.utxos[i].satoshis,
            script: Buffer.from(gatheredUtxos.utxos[i].scriptPk, "hex"),
          },
        });
      }
    }

    if (
      gatheredUtxos.totalAmount <
      finalFee * 2 + inscriptionSats + wasmDeploySize
    ) {
      throw new OylTransactionError(Error("Insufficient Balance"));
    }

    psbt.addOutput({
      value: finalFee + wasmDeploySize + 546,
      address: inscriberInfo.address,
    });

    const changeAmount =
      gatheredUtxos.totalAmount -
      (finalFee * 2 + wasmDeploySize + inscriptionSats);

    psbt.addOutput({
      address: account[account.spendStrategy.changeAddress].address,
      value: changeAmount,
    });

    const formattedPsbtTx = await formatInputsToSign({
      _psbt: psbt,
      senderPublicKey: account.taproot.pubkey,
      network: provider.network,
    });

    return { psbt: formattedPsbtTx.toBase64(), script };
  } catch (error) {
    throw error;
  }
};

export const deployCommit = async ({
  payload,
  utxos,
  account,
  provider,
  feeRate,
  signer,
}: {
  payload: AlkanesPayload;
  utxos: FormattedUtxo[];
  account: IAccount;
  provider: IOylProvider;
  feeRate?: number;
  signer: Signer;
}) => {
  if (!signer.taprootKeyPair) {
    throw new OylTransactionError(
      new Error("Signer does not have a taproot key pair")
    );
  }

  const tweakedTaprootKeyPair: bitcoin.Signer = tweakSigner(
    signer.taprootKeyPair,
    {
      network: provider.network,
    }
  );

  const tweakedPublicKey = tweakedTaprootKeyPair.publicKey.toString("hex");

  const { fee: commitFee } = await actualDeployCommitFee({
    payload,
    utxos,
    tweakedPublicKey,
    account,
    provider,
    feeRate,
  });

  const { psbt: finalPsbt, script } = await createDeployCommitPsbt({
    payload,
    utxos,
    tweakedPublicKey,
    account,
    provider,
    feeRate,
    fee: commitFee,
  });

  const { signedPsbt } = await signer.signAllInputs({
    rawPsbt: finalPsbt,
    finalize: true,
  });

  const result = await provider.pushPsbt({
    psbtBase64: signedPsbt,
  });

  return { ...result, script: script.toString("hex") };
};

export const createDeployRevealPsbt = async ({
  protostone,
  receiverAddress,
  script,
  feeRate,
  tweakedPublicKey,
  provider,
  fee = 0,
  commitTxId,
}: {
  protostone: Buffer;
  receiverAddress: string;
  script: Buffer;
  feeRate: number;
  tweakedPublicKey: string;
  provider: IOylProvider;
  fee?: number;
  commitTxId: string;
}) => {
  try {
    if (!feeRate) {
      feeRate = (await provider.esplora.getFeeEstimates())["1"];
    }

    const psbt: bitcoin.Psbt = new bitcoin.Psbt({ network: provider.network });
    const minFee = minimumFee({
      taprootInputCount: 1,
      nonTaprootInputCount: 0,
      outputCount: 2,
    });

    const revealTxBaseFee = minFee * feeRate < 250 ? 250 : minFee * feeRate;
    const revealTxChange = fee === 0 ? 0 : Number(revealTxBaseFee) - fee;

    const commitTxOutput = await getOutputValueByVOutIndex({
      txId: commitTxId,
      vOut: 0,
      esploraRpc: provider.esplora,
    });

    if (!commitTxOutput) {
      throw new OylTransactionError(new Error("Error getting vin #0 value"));
    }

    const p2pk_redeem = { output: script };

    const { output, witness } = bitcoin.payments.p2tr({
      internalPubkey: toXOnly(Buffer.from(tweakedPublicKey, "hex")),
      scriptTree: p2pk_redeem,
      redeem: p2pk_redeem,
      network: provider.network,
    });

    psbt.addInput({
      hash: commitTxId,
      index: 0,
      witnessUtxo: {
        value: commitTxOutput.value,
        script: output ?? Buffer.from(""),
      },
      tapLeafScript: [
        {
          leafVersion: LEAF_VERSION_TAPSCRIPT,
          script: p2pk_redeem.output,
          controlBlock: witness![witness!.length - 1],
        },
      ],
    });

    psbt.addOutput({
      value: 546,
      address: receiverAddress,
    });

    psbt.addOutput({
      value: 0,
      script: protostone,
    });

    if (revealTxChange > 546) {
      psbt.addOutput({
        value: revealTxChange,
        address: receiverAddress,
      });
    }

    return {
      psbt: psbt.toBase64(),
      fee: revealTxChange,
    };
  } catch (error) {
    throw error;
  }
};

export const deployReveal = async ({
  protostone,
  commitTxId,
  script,
  account,
  provider,
  feeRate,
  signer,
}: {
  protostone: Buffer;
  commitTxId: string;
  script: string;
  account: IAccount;
  provider: IOylProvider;
  feeRate?: number;
  signer: Signer;
}) => {
  if (!signer.taprootKeyPair) {
    throw new OylTransactionError(
      new Error("Signer does not have a taproot key pair")
    );
  }

  const tweakedTaprootKeyPair: bitcoin.Signer = tweakSigner(
    signer.taprootKeyPair,
    {
      network: provider.network,
    }
  );

  const tweakedPublicKey = tweakedTaprootKeyPair.publicKey.toString("hex");

  const { fee } = await actualTransactRevealFee({
    protostone,
    tweakedPublicKey,
    receiverAddress: account.taproot.address,
    commitTxId,
    script: Buffer.from(script, "hex"),
    provider,
    feeRate,
  });

  const { psbt: finalRevealPsbt } = await createTransactReveal({
    protostone,
    tweakedPublicKey,
    receiverAddress: account.taproot.address,
    commitTxId,
    script: Buffer.from(script, "hex"),
    provider,
    feeRate: feeRate ?? DEFAULT_FEE_RATE,
    fee,
  });

  let finalReveal = bitcoin.Psbt.fromBase64(finalRevealPsbt, {
    network: provider.network,
  });

  finalReveal.signInput(0, tweakedTaprootKeyPair);
  finalReveal.finalizeInput(0);

  const finalSignedPsbt = finalReveal.toBase64();

  const revealResult = await provider.pushPsbt({
    psbtBase64: finalSignedPsbt,
  });

  return revealResult;
};

export const actualTransactRevealFee = async ({
  protostone,
  tweakedPublicKey,
  commitTxId,
  receiverAddress,
  script,
  provider,
  feeRate,
}: {
  protostone: Buffer;
  tweakedPublicKey: string;
  commitTxId: string;
  receiverAddress: string;
  script: Buffer;
  provider: IOylProvider;
  feeRate?: number;
}) => {
  if (!feeRate) {
    feeRate = (await provider.esplora.getFeeEstimates())["1"] as number;
  }

  const { psbt } = await createTransactReveal({
    protostone,
    commitTxId,
    receiverAddress,
    script,
    tweakedPublicKey,
    provider,
    feeRate,
  });

  const { fee: estimatedFee } = await getEstimatedFee({
    feeRate,
    psbt,
    provider,
  });

  const { psbt: finalPsbt } = await createTransactReveal({
    protostone,
    commitTxId,
    receiverAddress,
    script,
    tweakedPublicKey,
    provider,
    feeRate,
    fee: estimatedFee,
  });

  const { fee: finalFee, vsize } = await getEstimatedFee({
    feeRate,
    psbt: finalPsbt,
    provider,
  });

  return { fee: finalFee, vsize };
};

export const actualExecuteFee = async ({
  alkanesUtxos,
  utxos,
  account,
  protostone,
  provider,
  feeRate,
  frontendFee,
  feeAddress,
}: {
  alkanesUtxos?: FormattedUtxo[];
  utxos: FormattedUtxo[];
  account: IAccount;
  protostone: Buffer;
  provider: IOylProvider;
  feeRate: number;
  frontendFee?: bigint;
  feeAddress?: string;
}) => {
  const { psbt } = await createExecutePsbt({
    alkanesUtxos,
    frontendFee,
    feeAddress,
    utxos,
    account,
    protostone,
    provider,
    feeRate,
  });

  const { fee: estimatedFee } = await getEstimatedFee({
    feeRate,
    psbt,
    provider,
  });

  const { psbt: finalPsbt } = await createExecutePsbt({
    alkanesUtxos,
    frontendFee,
    feeAddress,
    utxos,
    account,
    protostone,
    provider,
    feeRate,
    fee: estimatedFee,
  });

  const { fee: finalFee, vsize } = await getEstimatedFee({
    feeRate,
    psbt: finalPsbt,
    provider,
  });

  return { fee: finalFee, vsize };
};

export const executePsbt = async ({
  alkanesUtxos,
  utxos,
  account,
  protostone,
  provider,
  feeRate,
  frontendFee,
  feeAddress,
}: {
  alkanesUtxos?: FormattedUtxo[];
  utxos: FormattedUtxo[];
  account: IAccount;
  protostone: Buffer;
  provider: IOylProvider;
  feeRate?: number;
  frontendFee?: bigint;
  feeAddress?: string;
}) => {
  const { fee } = await actualExecuteFee({
    alkanesUtxos,
    frontendFee,
    feeAddress,
    utxos,
    account,
    protostone,
    provider,
    feeRate: feeRate ?? DEFAULT_FEE_RATE,
  });

  const { psbt: finalPsbt } = await createExecutePsbt({
    alkanesUtxos,
    frontendFee,
    feeAddress,
    utxos,
    account,
    protostone,
    provider,
    feeRate,
    fee,
  });

  return { psbt: finalPsbt, fee };
};

export const execute = async ({
  alkanesUtxos,
  utxos,
  account,
  protostone,
  provider,
  feeRate,
  signer,
  frontendFee,
  feeAddress,
}: {
  alkanesUtxos?: FormattedUtxo[];
  utxos: FormattedUtxo[];
  account: IAccount;
  protostone: Buffer;
  provider: IOylProvider;
  feeRate?: number;
  signer: Signer;
  frontendFee?: bigint;
  feeAddress?: string;
}) => {
  const { fee } = await actualExecuteFee({
    alkanesUtxos,
    frontendFee,
    feeAddress,
    utxos,
    account,
    protostone,
    provider,
    feeRate: feeRate ?? DEFAULT_FEE_RATE,
  });

  const { psbt: finalPsbt } = await createExecutePsbt({
    alkanesUtxos,
    frontendFee,
    feeAddress,
    utxos,
    account,
    protostone,
    provider,
    feeRate,
    fee,
  });

  const { signedPsbt } = await signer.signAllInputs({
    rawPsbt: finalPsbt,
    finalize: true,
  });

  const pushResult = await provider.pushPsbt({
    psbtBase64: signedPsbt,
  });

  return pushResult;
};

export const createTransactReveal = async ({
  protostone,
  receiverAddress,
  script,
  feeRate,
  tweakedPublicKey,
  provider,
  fee = 0,
  commitTxId,
}: {
  protostone: Buffer;
  receiverAddress: string;
  script: Buffer;
  feeRate: number;
  tweakedPublicKey: string;
  provider: IOylProvider;
  fee?: number;
  commitTxId: string;
}) => {
  try {
    if (!feeRate) {
      feeRate = (await provider.esplora.getFeeEstimates())["1"];
    }

    const psbt: bitcoin.Psbt = new bitcoin.Psbt({ network: provider.network });
    const minFee = minimumFee({
      taprootInputCount: 1,
      nonTaprootInputCount: 0,
      outputCount: 2,
    });

    const revealTxBaseFee = minFee * feeRate < 250 ? 250 : minFee * feeRate;
    const revealTxChange = fee === 0 ? 0 : Number(revealTxBaseFee) - fee;

    const commitTxOutput = await getOutputValueByVOutIndex({
      txId: commitTxId,
      vOut: 0,
      esploraRpc: provider.esplora,
    });

    if (!commitTxOutput) {
      throw new OylTransactionError(new Error("Error getting vin #0 value"));
    }

    const p2pk_redeem = { output: script };

    const { output, witness } = bitcoin.payments.p2tr({
      internalPubkey: toXOnly(Buffer.from(tweakedPublicKey, "hex")),
      scriptTree: p2pk_redeem,
      redeem: p2pk_redeem,
      network: provider.network,
    });

    psbt.addInput({
      hash: commitTxId,
      index: 0,
      witnessUtxo: {
        value: commitTxOutput.value,
        script: output ?? Buffer.from(""),
      },
      tapLeafScript: [
        {
          leafVersion: LEAF_VERSION_TAPSCRIPT,
          script: p2pk_redeem.output,
          controlBlock: witness![witness!.length - 1],
        },
      ],
    });

    psbt.addOutput({
      value: 546,
      address: receiverAddress,
    });

    psbt.addOutput({
      value: 0,
      script: protostone,
    });

    if (revealTxChange > 546) {
      psbt.addOutput({
        value: revealTxChange,
        address: receiverAddress,
      });
    }

    return {
      psbt: psbt.toBase64(),
      fee: revealTxChange,
    };
  } catch (error) {
    throw error;
  }
};
