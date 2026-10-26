import { useRef, useState, type ReactNode } from "react";
import { openOriginalOj } from "../ipc/contest";

export function ExternalSourceLink({ url, children }: { url: string; children: ReactNode }) {
  const inFlight = useRef(false);
  const [error, setError] = useState<string | null>(null);
  return <>
    <a href={url} rel="noreferrer noopener" target="_blank" onClick={(event) => {
      event.preventDefault();
      if (inFlight.current) return;
      inFlight.current = true;
      setError(null);
      void openOriginalOj(url).catch((cause: unknown) => {
        setError(String(cause).includes("unsafe_external_url")
          ? "链接无效，请检查原网址是否为有效的 HTTP 或 HTTPS 地址。"
          : "无法启动系统浏览器。可以重试，或复制下方网址到浏览器打开。");
      }).finally(() => { inFlight.current = false; });
    }}>{children}</a>
    {error ? <div role="alert"><p>{error}</p><p className="safe-note">{url}</p></div> : null}
  </>;
}
