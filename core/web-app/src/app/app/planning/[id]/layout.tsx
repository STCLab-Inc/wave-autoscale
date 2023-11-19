import React from 'react';

import PlanService from '@/services/plan';
import SubContentHeaderProps from '../../subcontent-header';
import PlanItemDrawerContainer from './plan-item-drawer-container';
import PlanningDetailTabs from './planning-detail-tabs';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import PlanningDetailControls from './planning-detail-controls';
import PlanningPage from '../page';
import PlanningDetailDrawer from '../../planning-drawer';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

async function getScalingPlanDefinition(dbId: string) {
  try {
    const scalingPlanDefinition = await PlanService.getPlan(dbId);
    return scalingPlanDefinition as ScalingPlanDefinitionEx;
  } catch (e) {
    console.error(e);
  }
}

const NEW_PATH = 'new';

export default async function PlanningDetailLayout({
  children,
  params: { id: dbId },
}: {
  children: React.ReactNode;
  params: { id: string };
}) {
  if (dbId === NEW_PATH) {
    return (
      <div className="relative flex h-full w-full">
        <PlanningPage />
        <PlanningDetailDrawer />
      </div>
    );
  }
  const scalingPlanDefinition = await getScalingPlanDefinition(dbId);
  if (!scalingPlanDefinition) {
    return (
      <div className="flex h-full w-full flex-row">
        <div className="flex h-full w-full flex-col">
          <SubContentHeaderProps title="Plan not found"></SubContentHeaderProps>
        </div>
      </div>
    );
  }
  return (
    <div className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <SubContentHeaderProps
          title={scalingPlanDefinition.metadata.title}
          right={<PlanningDetailControls />}
        ></SubContentHeaderProps>
        <PlanningDetailTabs />
        <div className="relative flex w-full flex-1 flex-col">{children}</div>
      </div>
      <PlanItemDrawerContainer />
    </div>
  );
}
