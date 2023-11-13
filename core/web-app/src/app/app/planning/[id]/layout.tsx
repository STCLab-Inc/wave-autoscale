import PlanService from '@/services/plan';
import ContentHeader from '../../content-header';
import PlanItemDrawerContainer from './plan-item-drawer-container';
import PlanningDetailTabs from './planning-detail-tabs';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import PlanningDetailControls from './planning-detail-controls';

async function getScalingPlanDefinition(dbId: string) {
  try {
    const scalingPlanDefinition = await PlanService.getPlan(dbId);
    return scalingPlanDefinition as ScalingPlanDefinition;
  } catch (e) {
    console.error(e);
  }
}

export default async function PlanningDetailLayout({
  children,
  params: { id: dbId },
}: {
  children: React.ReactNode;
  params: { id: string };
}) {
  const scalingPlanDefinition = await getScalingPlanDefinition(dbId);
  if (!scalingPlanDefinition) {
    return <div>Plan not found</div>;
  }
  return (
    <div className="flex h-full w-full flex-row">
      {/* Plan Detail Header */}
      <div className="flex flex-1 flex-col">
        <ContentHeader
          title={scalingPlanDefinition.kind}
          right={<PlanningDetailControls />}
        >
          <PlanningDetailTabs />
        </ContentHeader>
        <div className="relative w-full min-w-full flex-1">{children}</div>
      </div>
      <PlanItemDrawerContainer />
    </div>
  );
}
