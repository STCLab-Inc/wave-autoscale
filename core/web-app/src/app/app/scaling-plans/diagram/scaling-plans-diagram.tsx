'use client';

import React from 'react';
import { ReactFlowProvider } from 'reactflow';

import 'reactflow/dist/style.css';

import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

import PlanningDiagramFlow from './diagram-flow';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function ScalingPlansDiagram({
  scalingPlansItem,
  setScalingPlansItem,
  detailsModalFlag,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  scalingPlansItem?: ScalingPlanDefinitionEx | undefined;
  setScalingPlansItem: (plan: ScalingPlanDefinitionEx | undefined) => void;
  detailsModalFlag: boolean;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  return (
    <div className="h-full w-full">
      <ReactFlowProvider>
        <PlanningDiagramFlow
          scalingPlansItem={scalingPlansItem}
          setScalingPlansItem={setScalingPlansItem}
          detailsModalFlag={detailsModalFlag}
          setDetailsModalFlag={setDetailsModalFlag}
          setFetchFlag={setFetchFlag}
        />
      </ReactFlowProvider>
    </div>
  );
}
