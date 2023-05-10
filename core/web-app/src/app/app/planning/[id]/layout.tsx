import ContentHeader from '../../content-header';
import PlanningDetailTabs from './tabs';

export default function PlanningDetailLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      {/* Plan Detail Header */}
      <ContentHeader title="Free Plan Tenancy Autoscaling">
        <PlanningDetailTabs />
      </ContentHeader>
      <div className="w-full min-w-full flex-1">{children}</div>
    </>
  );
}
