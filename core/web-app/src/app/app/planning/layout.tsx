export default function PlanningLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-full w-full">
      <aside className="flex w-80 flex-col border-r border-base-300 bg-base-200">
        {/* Header of aside */}
        <div className="prose flex h-14 w-full items-center justify-start border-b border-base-300 px-3">
          <h4 className="m-0 flex-1">Plans</h4>
          <span className="badge">4</span>
        </div>
        {/* Plan List */}
        <div className="flex-1 overflow-y-auto">{/* Plan Item */}</div>
      </aside>
      <main className="flex flex-1 flex-col">{children}</main>
    </div>
  );
}
