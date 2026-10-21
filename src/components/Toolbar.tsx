interface ToolbarProps {
  url: string;
  onUrlChange: (url: string) => void;
  bootedCount: number;
  busy: boolean;
  onOpenAll: () => void;
  onShutdownAll: () => void;
  onRefresh: () => void;
}

export function Toolbar(props: ToolbarProps) {
  const { url, onUrlChange, bootedCount, busy, onOpenAll, onShutdownAll, onRefresh } = props;
  const noBooted = bootedCount === 0;

  return (
    <div className="toolbar">
      <form
        className="url-form"
        onSubmit={(e) => {
          e.preventDefault();
          onOpenAll();
        }}
      >
        <input
          className="input"
          type="text"
          placeholder="https://… or myapp://deep/link"
          value={url}
          spellCheck={false}
          autoCorrect="off"
          autoCapitalize="off"
          onChange={(e) => onUrlChange(e.target.value)}
        />
        <button className="btn primary" type="submit" disabled={busy || noBooted || !url.trim()}>
          Open on all booted
        </button>
      </form>
      <div className="toolbar-row">
        <span className="muted">{bootedCount} booted</span>
        <div className="spacer" />
        <button className="btn" onClick={onShutdownAll} disabled={busy || noBooted}>
          Shutdown all
        </button>
        <button className="btn icon" onClick={onRefresh} title="Refresh" aria-label="Refresh">
          ↻
        </button>
      </div>
    </div>
  );
}
