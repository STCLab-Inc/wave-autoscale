export default function StatusBadge({ success }: { success: boolean }) {
  if (success) {
    return <div className="badge-success badge p-2 text-xs">Success</div>;
  }
  return <div className="badge-error badge p-2 text-xs">Fail</div>;
}
