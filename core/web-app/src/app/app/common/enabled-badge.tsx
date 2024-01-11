export default function EnabledBadge({ enabled }: { enabled: boolean }) {
  if (enabled) {
    return <div className="badge-success badge p-2 text-xs">Enabled</div>;
  }
  return <div className="badge-error badge p-2 text-xs">Disabled</div>;
}
