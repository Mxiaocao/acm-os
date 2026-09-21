import { useEffect, useState, useRef } from "react";
import { displayPath } from "./display-path";
import { createVault, deleteVault, getVaultManagerStatus, openVault, previewDeleteVault, vaultErrorMessage, type DeletePreview, type ManagerStatus } from "../ipc/vault-manager";

export function VaultManagerPanel() {
  const [status, setStatus] = useState<ManagerStatus | null>(null);
  const [name, setName] = useState("");
  const [rootPath, setRootPath] = useState("");
  const submitting = useRef(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [recoveryPath, setRecoveryPath] = useState("");
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
    setBusy(true); setError(""); setMessage(""); setRecoveryPath("");
    try { await action(); } catch (cause) { setError(vaultErrorMessage(cause)); }
    finally { submitting.current = false; setBusy(false); }
  }
  const current = status?.current;
  return <section aria-labelledby="vault-manager-heading" className="content-panel vault-manager" data-testid="vault-manager">
    <header className="vault-manager-header">
      <div><h2 id="vault-manager-heading">Vault 工作区</h2><p>在不同工作区之间切换，比赛、题目和奖励各自独立。</p></div>
      <span className="vault-count">{status ? `${status.vaults.length} 个工作区` : "读取中"}</span>
    </header>
    <div className="vault-current" aria-live="polite" data-testid="current-vault">
      <div className="vault-current-heading"><span>当前工作区</span>{current && current.state !== "none" && <span className={`vault-state${current.state === "ready" ? "" : " vault-state--warning"}`}>{current.state === "ready" ? "使用中" : "不可用"}</span>}</div>
      {!status ? <p>{error ? "暂时无法读取工作区" : "正在读取 Vault…"}</p> : current?.state === "none" ? <><h3>还没有打开工作区</h3><p>新建一个 Vault，或从下方列表打开。</p></> : current && <>
        <h3>{current.vault.name}</h3><p className="vault-path">{displayPath(current.vault.rootPath)}</p>
        {current.state !== "ready" && <p>请恢复原路径，或打开其他工作区。</p>}
      </>}
    </div>
    {error && <p className="error-copy" role="alert">{error}</p>}
    {message && <p className="status-message" role="status">{message}</p>}
    {recoveryPath && <details className="vault-recovery-details">
      <summary>查看保留文件的位置</summary>
      <p className="vault-path">{displayPath(recoveryPath)}</p>
    </details>}
    <fieldset className="vault-controls" disabled={busy || !status}>
      <legend className="sr-only">管理工作区</legend>
      <details className="vault-create"><summary>新建 Vault<span>使用一个独立的文件夹</span></summary>
      <form className="workspace-form vault-create-form" onSubmit={event => { event.preventDefault(); void run(async () => {
        const created = await createVault({ name, rootPath });
        setStatus(await getVaultManagerStatus()); setName(""); setRootPath(""); setMessage(`已创建 ${created.name}，请从下方列表打开。`);
      }); }}>
        <label htmlFor="vault-name">名称
        <input id="vault-name" value={name} onChange={event => setName(event.target.value)} required maxLength={80} placeholder="例如：算法练习" /></label>
        <label htmlFor="vault-path">文件夹路径
        <input id="vault-path" value={rootPath} onChange={event => setRootPath(event.target.value)} required placeholder="例如：E:\Vaults\Practice" aria-describedby="vault-path-help" /></label>
        <p id="vault-path-help" className="vault-help">填写一个已存在的文件夹。创建后，再从列表打开。</p><button className="primary-action" type="submit">创建 Vault</button>
      </form></details>
      <h3 className="vault-list-heading">全部工作区</h3>
      {status?.vaults.length === 0 && <p>暂无 Vault。</p>}
      <ul className="vault-list">{status?.vaults.map(vault => <li key={vault.id} data-vault-id={vault.id}>
        <div className="vault-list-info"><div className="vault-list-title"><strong>{vault.name}</strong><span className="vault-list-status">{vault.isCurrent ? current?.state === "ready" ? "当前" : "不可用" : "未打开"}</span></div>
        <p className="vault-path">{displayPath(vault.rootPath)}</p></div><div className="vault-list-actions">
        <button className="secondary-action" type="button" disabled={vault.isCurrent && current?.state === "ready"} onClick={() => void run(async () => {
          setPreview(null); setStatus(await openVault({ vaultId: vault.id }));
        })}>打开<span className="sr-only"> {vault.name}</span></button>
        <button className="vault-delete-action" title={vault.isCurrent ? "请先切换到其他工作区再删除" : undefined} type="button" disabled={vault.isCurrent} onClick={() => void run(async () => {
          setConfirmation(""); setPreview(await previewDeleteVault({ vaultId: vault.id }));
        })}>删除<span className="sr-only"> {vault.name}</span></button></div>
      </li>)}</ul>
    </fieldset>
    {preview && <section className="vault-delete-confirm workspace-form" role="dialog" aria-modal="false" aria-labelledby="vault-delete-title">
      <h3 id="vault-delete-title">再次确认删除 {preview.name}</h3>
      <p>将从列表移除该工作区。笔记不会删除，数据库和备份会保留在以下位置：</p><p className="vault-path">{displayPath(preview.retainedDataPath)}</p>
      <label htmlFor="vault-delete-confirmation">输入 {preview.confirmationText}</label>
      <input id="vault-delete-confirmation" autoFocus disabled={busy} value={confirmation} onChange={event => setConfirmation(event.target.value)} />
      <button className="danger-action" type="button" disabled={busy || confirmation !== preview.confirmationText} onClick={() => void run(async () => {
        const pending = preview;
        setPreview(null);
        setStatus(await deleteVault({ vaultId: pending.vaultId, token: pending.token, confirmation }));
        setMessage(`已移除「${pending.name}」，笔记、数据库和备份已保留。`);
        setRecoveryPath(pending.retainedDataPath);
      })}>确认删除</button>
      <button className="secondary-action" type="button" disabled={busy} onClick={() => setPreview(null)}>取消</button>
    </section>}
  </section>;
}
