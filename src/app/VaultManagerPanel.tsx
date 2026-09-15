import { useEffect, useState, useRef } from "react";
import { createVault, deleteVault, getVaultManagerStatus, openVault, previewDeleteVault, vaultErrorMessage, type DeletePreview, type ManagerStatus } from "../ipc/vault-manager";

export function VaultManagerPanel() {
  const [status, setStatus] = useState<ManagerStatus | null>(null);
  const [name, setName] = useState("");
  const [rootPath, setRootPath] = useState("");
  const submitting = useRef(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [preview, setPreview] = useState<DeletePreview | null>(null);
  const [confirmation, setConfirmation] = useState("");
  useEffect(() => {
    let live = true;
    getVaultManagerStatus().then(value => { if (live) setStatus(value); }).catch(cause => { if (live) setError(vaultErrorMessage(cause)); });
    return () => { live = false; };
  }, []);
  async function run(action: () => Promise<void>) {
    if (submitting.current) return;
    submitting.current = true;
    setBusy(true); setError(""); setMessage("");
    try { await action(); } catch (cause) { setError(vaultErrorMessage(cause)); }
    finally { submitting.current = false; setBusy(false); }
  }
  const current = status?.current;
  return <section aria-labelledby="vault-manager-heading" className="content-panel" data-testid="vault-manager">
    <h2 id="vault-manager-heading">Vault Manager / 工作区</h2>
    <p>每个 Vault 使用独立数据库。Legacy Data 保持原样。</p>
    <div aria-live="polite" data-testid="current-vault">
      <h3>Current Vault / 当前 Vault</h3>
      {!status ? <p>正在读取 Vault…</p> : current?.state === "none" ? <p>尚未打开 Vault</p> : current && <>
        <strong>{current.vault.name}</strong><p>{current.vault.rootPath}</p>
        <p>{current.state === "ready" ? "已打开" : "不可用：请恢复原路径或打开其他 Vault"}</p>
      </>}
    </div>
    {error && <p role="alert">{error}</p>}
    {message && <p role="status">{message}</p>}
    <fieldset disabled={busy || !status}>
      <legend>Create Vault / 创建 Vault</legend>
      <form onSubmit={event => { event.preventDefault(); void run(async () => {
        const created = await createVault({ name, rootPath });
        setStatus(await getVaultManagerStatus()); setName(""); setRootPath(""); setMessage(`已创建 ${created.name}，点击 Open 打开。`);
      }); }}>
        <label htmlFor="vault-name">Vault 名称</label>
        <input id="vault-name" value={name} onChange={event => setName(event.target.value)} required maxLength={80} />
        <label htmlFor="vault-path">Vault 文件夹路径（已存在）</label>
        <input id="vault-path" value={rootPath} onChange={event => setRootPath(event.target.value)} required />
        <button type="submit">Create / 创建</button>
      </form>
      <h3>Vault List / Vault 列表</h3>
      {status?.vaults.length === 0 && <p>暂无 Vault。</p>}
      <ul>{status?.vaults.map(vault => <li key={vault.id} data-vault-id={vault.id}>
        <strong>{vault.name}</strong> — <span>{vault.isCurrent ? current?.state === "ready" ? "当前" : "不可用" : "未打开"}</span>
        <p>{vault.rootPath}</p>
        <button type="button" disabled={vault.isCurrent && current?.state === "ready"} onClick={() => void run(async () => {
          setPreview(null); setStatus(await openVault({ vaultId: vault.id }));
        })}>Open / 打开 {vault.name}</button>
        <button type="button" disabled={vault.isCurrent} onClick={() => void run(async () => {
          setConfirmation(""); setPreview(await previewDeleteVault({ vaultId: vault.id }));
        })}>Delete / 删除 {vault.name}</button>
      </li>)}</ul>
    </fieldset>
    {preview && <section role="dialog" aria-modal="false" aria-labelledby="vault-delete-title">
      <h3 id="vault-delete-title">再次确认删除 {preview.name}</h3>
      <p>将从列表移除该 Vault。笔记保留，数据库及备份保留在：</p><p>{preview.retainedDataPath}</p>
      <label htmlFor="vault-delete-confirmation">输入 {preview.confirmationText}</label>
      <input id="vault-delete-confirmation" autoFocus disabled={busy} value={confirmation} onChange={event => setConfirmation(event.target.value)} />
      <button type="button" disabled={busy || confirmation !== preview.confirmationText} onClick={() => void run(async () => {
        const pending = preview;
        setPreview(null);
        setStatus(await deleteVault({ vaultId: pending.vaultId, token: pending.token, confirmation }));
        setMessage(`已移除 ${pending.name}；恢复目录：${pending.retainedDataPath}`);
      })}>Confirm Delete / 确认删除</button>
      <button type="button" disabled={busy} onClick={() => setPreview(null)}>Cancel / 取消</button>
    </section>}
  </section>;
}
