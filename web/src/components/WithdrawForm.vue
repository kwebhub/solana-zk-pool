<script setup lang="ts">
import { ref, computed } from "vue";
import { storeToRefs } from "pinia";
import { useWalletStore } from "@/stores/wallet";
import { useWithdraw } from "@/composables/useWithdraw";

const walletStore = useWalletStore();
const { isConnected } = storeToRefs(walletStore);

const { loading, error, txSignature, proofGenerating, withdraw, reset } = useWithdraw();

const nullifierNote = ref("");
const recipient = ref("");
const amount = ref(0.001);

const isValid = computed(() => {
  return (
    nullifierNote.value.trim().length > 0 && recipient.value.trim().length > 0 && isConnected.value
  );
});

const handleWithdraw = async () => {
  if (!isValid.value) return;

  try {
    const lines = nullifierNote.value.split("\n");
    const nullifierSecretHex = lines[0]?.replace("Nullifier Secret: ", "").trim() || "";
    const secretHex = lines[1]?.replace("Secret: ", "").trim() || "";
    const noteSecretHex = lines[2]?.replace("Note Secret: ", "").trim() || "";

    await withdraw(nullifierSecretHex, secretHex, noteSecretHex, amount.value, recipient.value);
  } catch (err) {
    console.error("Withdraw execution error:", err);
  }
};

const handleReset = () => {
  reset();
  nullifierNote.value = "";
  recipient.value = "";
  amount.value = 0.001;
};
</script>

<template lang="pug">
.withdraw-container
  h2 Withdraw SOL
  .form-group
    label Deposit Note (Paste full note text)
    textarea(
      v-model="nullifierNote"
      :disabled="loading || !isConnected"
      rows="5"
      placeholder="Paste your saved note here..."
    )
  .form-group
    label Recipient Address
    input(
      type="text"
      v-model="recipient"
      :disabled="loading || !isConnected"
      placeholder="Solana Base58 Address"
    )
  .form-group
    label Amount (SOL)
    input(
      type="number"
      v-model="amount"
      :disabled="loading || !isConnected"
      :step="0.001"
      :min="0.001"
    )

  button.withdraw-btn(
    @click="handleWithdraw"
    :disabled="!isValid || loading"
    :class="{ loading }"
  )
    span(v-if="proofGenerating") Generating ZK Proof...
    span(v-else-if="loading") Processing Tx...
    span(v-else) Withdraw

  .error(v-if="error") {{ error }}
  .success(v-if="txSignature")
    span Success! Tx:
    a(
      :href="`https://explorer.solana.com/tx/${txSignature}?cluster=devnet`"
      target="_blank"
    ) {{ txSignature.slice(0, 8) }}...{{ txSignature.slice(-8) }}

  button.reset-btn(v-if="txSignature" @click="handleReset") New Withdraw
</template>

<style scoped lang="scss">
.withdraw-container {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  max-width: 480px;
  margin: 20px auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

h2 {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
  color: #0f172a;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;

  label {
    font-size: 14px;
    font-weight: 500;
    color: #334155;
  }
}

textarea,
input {
  padding: 10px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  font-size: 14px;
  font-family: monospace;
  transition: border-color 0.2s;

  &:focus {
    outline: none;
    border-color: #4f46e5;
  }

  &:disabled {
    background: #f1f5f9;
    cursor: not-allowed;
  }
}

.withdraw-btn {
  background: #4f46e5;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;

  &:hover:not(:disabled) {
    background: #4338ca;
  }

  &:disabled {
    background: #cbd5e1;
    color: #94a3b8;
    cursor: not-allowed;
  }

  &.loading {
    opacity: 0.7;
    cursor: wait;
  }
}

.error {
  color: #dc2626;
  font-size: 13px;
}

.success {
  color: #16a34a;
  font-size: 13px;

  a {
    color: #16a34a;
    margin-left: 4px;
  }
}

.reset-btn {
  background: transparent;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 8px;
  font-size: 13px;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: #f1f5f9;
    color: #0f172a;
  }
}
</style>
