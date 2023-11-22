'use client';

import React from 'react';
import { ReactFlowProvider } from 'reactflow';

import 'reactflow/dist/style.css';

import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

import PlanningDiagramFlow from './diagram-flow';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function PlanningDiagramComponent({
  plansItem,
  setPlansItem,
  detailsModalFlag,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  plansItem?: ScalingPlanDefinitionEx | undefined;
  setPlansItem: (plan: ScalingPlanDefinitionEx | undefined) => void;
  detailsModalFlag: boolean;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  return (
    <div className="h-full w-full">
      <ReactFlowProvider>
        <PlanningDiagramFlow
          plansItem={plansItem}
          setPlansItem={setPlansItem}
          detailsModalFlag={detailsModalFlag}
          setDetailsModalFlag={setDetailsModalFlag}
          setFetchFlag={setFetchFlag}
        />
      </ReactFlowProvider>
    </div>
  );
}
