import ContentHeader from '../../content-header';
import PlanDrawerContainer from './plan-drawer-container';
import PlanningDetailTabs from './tabs';

export default function PlanningDetailLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex w-full flex-row h-full">
      {/* Plan Detail Header */}
      <div className="flex flex-1 flex-col">
        <ContentHeader title="Free Plan Tenancy Autoscaling">
          <PlanningDetailTabs />
        </ContentHeader>
        <div className="relative w-full min-w-full flex-1">{children}</div>
      </div>
      <PlanDrawerContainer />
    </div>
  );
}
