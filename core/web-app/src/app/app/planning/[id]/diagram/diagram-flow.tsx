'use client';

import {
  Background,
  BackgroundVariant,
  Controls,
  NodeChange,
  ReactFlow,
} from 'reactflow';
import { usePlanStore } from '../../plan-store';
import { useEffect, useMemo, useState } from 'react';
import { produce } from 'immer';
import { useParams } from 'next/navigation';

const POSITION_X_OFFSET = 200;

export default function PlanningDiagramFlow() {
  const { id: scalingPlanId } = useParams();
  const plans = usePlanStore(
    (state) => state.currentScalingPlanState?.scalingPlan?.plans || []
  );
  const metricIds = usePlanStore(
    (state) => state.currentScalingPlanState?.metricIds || []
  );
  console.log({ metricIds2: metricIds });
  const updatePlanItemUI = usePlanStore((state) => state.updatePlanItemUI);


  const nodes = useMemo(() => {
    const planNodes = plans.map((plan, index) => {
      console.log({ plan });
      return {
        // Default properties
        id: plan.id,
        position: { x: POSITION_X_OFFSET * index, y: 100 },
        data: { label: plan.id },
        draggable: false,
        // Get the ui property from the plan
        ...plan?.ui,
      };
    });
    const metricNodes = metricIds.map((metricId, index) => {
      return {
        // Default properties
        id: metricId,
        position: { x: POSITION_X_OFFSET * index, y: 0 },
        data: { label: metricId },
        draggable: false,
      };
    });
    console.log({ metricIds });
    console.log({ metricNodes });
    return [...planNodes, ...metricNodes];
  }, [plans, metricIds]);

  const onNodesChange = (nodes: NodeChange[]) => {
    // Find the plan by id
    const findPlan = (planId: string) =>
      plans.find((plan) => plan.id === planId);

    // Update the plan with new attributes
    nodes.forEach((node) => {
      const type = node.type;
      // Update the plan with the new position
      if (type === 'position' && node.position) {
        const plan = findPlan(node.id);
        if (plan) {
          const newPlan = produce(plan, (draft) => {
            draft.ui = { ...draft.ui, position: node.position };
          });
          updatePlanItemUI(newPlan);
        }
      } else if (type === 'select') {
        // Update the plan with the new selected state
        const plan = findPlan(node.id);
        if (plan) {
          const newPlan = produce(plan, (draft) => {
            draft.ui = { ...draft.ui, selected: node.selected };
          });
          updatePlanItemUI(newPlan);
        }
      }
    });
  };

  return (
    <ReactFlow nodes={nodes} onNodesChange={onNodesChange}>
      <Background color="#ccc" variant={BackgroundVariant.Dots} />
      <Controls />
    </ReactFlow>
  );
}
