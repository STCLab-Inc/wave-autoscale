export default function EnabledBadge({ enabled }: { enabled: boolean }) {
  if (enabled) {
    return (
      <div className="badge-success badge h-6 w-16 rounded-md text-xs">
        Enabled
      </div>
    );
  }
  return (
    <div className="badge-disabled badge h-6 w-16 rounded-md text-xs">
      Disabled
    </div>
  );
}
