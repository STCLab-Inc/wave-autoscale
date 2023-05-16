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

export default function PlanningDiagramFlow() {
  const { id: scalingPlanId } = useParams();
  const plans = usePlanStore(
    (state) => state.currentScalingPlanState?.scalingPlan?.plans || []
  );
  const sync = usePlanStore((state) => state.fetch);
  const updatePlanItemUI = usePlanStore((state) => state.updatePlanItemUI);

  // If scalingPlanId changes, fetch the scaling plan then it updates plans and nodes.
  useEffect(() => {
    const fetch = async () => {
      await sync(scalingPlanId);
    };
    fetch();
  }, [scalingPlanId]);

  const nodes = useMemo(() => {
    return plans.map((plan) => {
      console.log({ plan });
      return {
        // Default properties
        id: plan.id,
        position: { x: 0, y: 0 },
        data: { label: plan.id },
        draggable: false,
        // Get the ui property from the plan
        ...plan?.ui,
      };
    });
  }, [plans]);

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
          updatePlanItemUI(scalingPlanId, newPlan);
        }
      } else if (type === 'select') {
        // Update the plan with the new selected state
        const plan = findPlan(node.id);
        if (plan) {
          const newPlan = produce(plan, (draft) => {
            draft.ui = { ...draft.ui, selected: node.selected };
          });
          updatePlanItemUI(scalingPlanId, newPlan);
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
