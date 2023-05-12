'use client';

import {
  Background,
  BackgroundVariant,
  Controls,
  NodeChange,
  ReactFlow,
  useNodes,
  useReactFlow,
} from 'reactflow';
import { usePlanStore } from '../plan-store';
import { useEffect, useMemo } from 'react';
import { produce } from 'immer';

export default function PlanningDiagramFlow() {
  const plans = usePlanStore((state) => state.plans);
  const updatePlan = usePlanStore((state) => state.updatePlan);

  const nodes = useMemo(() => {
    return plans.map((plan) => {
      console.log({ plan });
      return {
        // Default properties
        id: plan.id,
        position: { x: 0, y: 0 },
        data: { label: plan.id },
        draggable: true,
        // Get the ui property from the plan
        ...plan?.ui,
      };
    });
  }, [plans]);

  const onNodesChange = (nodes: NodeChange[]) => {
    nodes.forEach((node) => {
      const type = node.type;
      if (type === 'position' && node.position) {
        const plan = plans.find((plan) => plan.id === node.id);
        if (plan) {
          // Update the plan with the new position
          const newPlan = produce(plan, (draft) => {
            draft.ui = { ...draft.ui, position: node.position };
          });
          updatePlan(newPlan);
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
