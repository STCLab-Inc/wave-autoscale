import React from 'react';

import PlanService from '@/services/plan';
import SubContentHeader from '../../subcontent-header';
import PlanItemDrawerContainer from './plan-item-drawer-container';
import PlanningDetailTabs from './planning-detail-tabs';
import PlanningDetailControls from './planning-detail-controls';
import PlanningPage from '../page';
import PlanningDetailDrawer from '../../planning-drawer';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

async function getScalingPlanDefinition(dbId: string) {
  try {
    const scalingPlanDefinition = await PlanService.getPlan(dbId);
    return scalingPlanDefinition as ScalingPlanDefinitionEx;
  } catch (error) {
    console.error(error);
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
          <SubContentHeader title="Plan not found"></SubContentHeader>
        </div>
      </div>
    );
  }
  return (
    <div className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <SubContentHeader
          title={scalingPlanDefinition.metadata.title}
          right={<PlanningDetailControls />}
        ></SubContentHeader>
        <PlanningDetailTabs />
        <div className="relative flex w-full flex-1 flex-col">{children}</div>
      </div>
      <PlanItemDrawerContainer />
    </div>
  );
}
