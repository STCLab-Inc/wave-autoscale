'use client';

import React from 'react';
import { ReactFlowProvider } from 'reactflow';
import PlanningDiagramFlow from './diagram-flow';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import 'reactflow/dist/style.css';

export default function ScalingPlanDiagram({
  scalingPlan,
}: {
  scalingPlan: ScalingPlanDefinition | undefined;
}) {
  return (
    <div className="h-full w-full">
      <ReactFlowProvider>
        <PlanningDiagramFlow scalingPlan={scalingPlan} />
      </ReactFlowProvider>
    </div>
  );
}
