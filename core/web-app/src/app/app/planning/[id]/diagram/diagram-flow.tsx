'use client';

import {
  Background,
  BackgroundVariant,
  Controls,
  Edge,
  MarkerType,
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
  const metricIds = usePlanStore(
    (state) => state.currentScalingPlanState?.metricIds || []
  );
  const scalingComponentIds = usePlanStore(
    (state) => state.currentScalingPlanState?.scalingComponentIds || []
  );
  console.log({ metricIds2: metricIds });
  const updatePlanItemUI = usePlanStore((state) => state.updatePlanItemUI);

  // Nodes are the plans and metrics
  const nodes = useMemo(() => {
    const POSITION_X_OFFSET = 200;
    const POSITION_Y_OFFSET = 200;
    const maxWidth =
      Math.max(plans?.length, metricIds?.length, scalingComponentIds?.length) *
      POSITION_X_OFFSET;

    const planNodeXOffset = (maxWidth - plans?.length * POSITION_X_OFFSET) / 2;
    const planNodes = plans.map((plan, index) => {
      return {
        // Default properties
        id: plan.id,
        type: 'default',
        position: {
          x: POSITION_X_OFFSET * index + planNodeXOffset,
          y: POSITION_Y_OFFSET,
        },
        data: { label: plan.id },
        draggable: false,
        // Get the ui property from the plan
        ...plan?.ui,
      };
    });
    const metricNodeXOffset =
      (maxWidth - metricIds?.length * POSITION_X_OFFSET) / 2;
    const metricNodes = metricIds.map((metricId, index) => {
      return {
        // Default properties
        id: metricId,
        type: 'input',
        position: { x: POSITION_X_OFFSET * index + metricNodeXOffset, y: 0 },
        data: { label: metricId },
        draggable: false,
      };
    });

    const scalingComponentNodeXOffset =
      (maxWidth - scalingComponentIds?.length * POSITION_X_OFFSET) / 2;
    const scalingComponentNodes = scalingComponentIds.map(
      (scalingComponentId, index) => {
        return {
          // Default properties
          id: scalingComponentId,
          type: 'outpput',
          position: {
            x: POSITION_X_OFFSET * index + scalingComponentNodeXOffset,
            y: POSITION_Y_OFFSET * 2,
          },
          data: { label: scalingComponentId },
          draggable: false,
        };
      }
    );

    return [...planNodes, ...metricNodes, ...scalingComponentNodes];
  }, [plans, metricIds]);

  const edges = useMemo(() => {
    const edgesForPlanAndMetric: Edge[] = [];
    const edgesForPlanAndScalingComponent: Edge[] = [];
    plans.forEach((plan) => {
      const metricIds = plan?.ui?.metricIds;
      if (metricIds?.length) {
        metricIds?.forEach((metricId: string) => {
          edgesForPlanAndMetric.push({
            id: `${plan.id}-${metricId}`,
            source: metricId,
            target: plan.id,
            animated: true,
            type: 'smoothstep',
            markerEnd: {
              type: MarkerType.ArrowClosed,
              width: 20,
              height: 20,
            },
            // arrowHeadType: 'arrowclosed',
          });
        });
      }

      const scalingComponentIds = plan?.ui?.scalingComponentIds;
      if (scalingComponentIds?.length) {
        scalingComponentIds?.forEach((scalingComponentId: string) => {
          edgesForPlanAndMetric.push({
            id: `${plan.id}-${scalingComponentId}`,
            source: plan.id,
            target: scalingComponentId,
            animated: true,
            type: 'smoothstep',
            markerEnd: {
              type: MarkerType.ArrowClosed,
              width: 20,
              height: 20,
            },
            // arrowHeadType: 'arrowclosed',
          });
        });
      }
    });
    return [...edgesForPlanAndMetric, ...edgesForPlanAndScalingComponent];
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
    <ReactFlow
      nodes={nodes}
      edges={edges}
      fitView={true}
      fitViewOptions={{
        padding: 0.5,
      }}
      onNodesChange={onNodesChange}
    >
      <Background color="#ccc" variant={BackgroundVariant.Dots} />
      <Controls />
    </ReactFlow>
  );
}
