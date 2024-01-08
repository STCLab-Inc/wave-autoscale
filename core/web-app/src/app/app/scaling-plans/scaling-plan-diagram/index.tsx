'use client';

import React from 'react';
import { ReactFlowProvider } from 'reactflow';
import PlanningDiagramFlow from './diagram-flow';
import 'reactflow/dist/style.css';
import { ScalingPlanDefinitionEx } from '../scaling-plan-definition-ex';

export default function ScalingPlanDiagram({
  scalingPlan,
}: {
  scalingPlan: ScalingPlanDefinitionEx;
}) {
  return (
    <div className="h-full w-full">
      <ReactFlowProvider>
        <PlanningDiagramFlow scalingPlan={scalingPlan} />
      </ReactFlowProvider>
    </div>
  );
}
