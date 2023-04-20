import PlanningDetailTabs from './tabs';

export default function PlanningDetailLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      {/* Plan Detail Header */}
      <div className="h-22 prose flex w-full min-w-full flex-col border-b border-base-300 bg-slate-50 px-3 pt-3">
        {/* Plan Title */}
        <h3 className="m-3 flex-1">Free Plan Tenancy Autoscaling</h3>
        <PlanningDetailTabs />
      </div>
      <div className="w-full min-w-full flex-1">{children}</div>
    </>
  );
}
