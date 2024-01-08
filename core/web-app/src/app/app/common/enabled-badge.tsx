export default function EnabledBadge({ enabled }: { enabled: boolean }) {
  if (enabled) {
    return <div className="badge-success badge p-3">Enabled</div>;
  }
  return <div className="badge-error badge p-3">Disabled</div>;
}
