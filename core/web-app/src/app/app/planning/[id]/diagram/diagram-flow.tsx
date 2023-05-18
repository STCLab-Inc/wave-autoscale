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
import { MetricUI, ScalingComponentUI, usePlanStore } from '../../plan-store';
import { useEffect, useMemo, useState } from 'react';
import { produce } from 'immer';
import { useParams } from 'next/navigation';
import PlanNode from './plan-node';
import MetricNode from './metric-node';
import ScalingComponentNode from './scaling-component-node';
import MetricService from '@/services/metric';
import ScalingComponentService from '@/services/scaling-component';
import { flatten, keyBy, unionBy } from 'lodash';

const nodeTypes = {
  plan: PlanNode,
  metric: MetricNode,
  scalingComponent: ScalingComponentNode,
};

export default function PlanningDiagramFlow() {
  const plans = usePlanStore(
    (state) => state.currentScalingPlanState?.scalingPlan?.plans || []
  );
  // const metricIds = usePlanStore(
  //   (state) => state.currentScalingPlanState?.metricIds || []
  // );
  // const scalingComponentIds = usePlanStore(
  //   (state) => state.currentScalingPlanState?.scalingComponentIds || []
  // );
  const [metricMap, setMetricMap] = useState<any>({});
  const [scalingComponentMap, setScalingComponentMap] = useState<any>({});
  const updatePlanItemUI = usePlanStore((state) => state.updatePlanItemUI);

  // Initialize
  useEffect(() => {
    const fetch = async () => {
      const promises = [
        MetricService.getMetrics(),
        ScalingComponentService.getScalingComponents(),
      ];
      const results = await Promise.all(promises);
      console.log({ results });
      setMetricMap(keyBy(results[0], 'id'));
      setScalingComponentMap(keyBy(results[1], 'id'));
    };
    fetch();
  }, []);

  // Nodes are the plans and metrics
  const nodes = useMemo(() => {
    const POSITION_X_OFFSET = 400;
    const POSITION_Y_OFFSET = 200;
    const metrics = unionBy(
      flatten(plans?.map((plan) => plan?.ui?.metrics)),
      'id'
    );
    const scalingComponents = unionBy(
      flatten(plans?.map((plan) => plan?.ui?.scalingComponents)),
      'id'
    );

    const maxWidth =
      Math.max(plans?.length, metrics?.length, scalingComponents?.length) *
      POSITION_X_OFFSET;

    const planNodeXOffset = (maxWidth - plans?.length * POSITION_X_OFFSET) / 2;
    const planNodes = plans.map((plan, index) => {
      return {
        // Default properties
        id: plan.id,
        type: 'plan',
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
      (maxWidth - metrics?.length * POSITION_X_OFFSET) / 2;

    const metricNodes = metrics.map((metric: MetricUI, index) => {
      console.log('metricsss', metric);
      return {
        // Default properties
        id: metric.id,
        type: 'metric',
        position: { x: POSITION_X_OFFSET * index + metricNodeXOffset, y: 0 },
        data: {
          label: metric.id,
          metricKind: metricMap[metric.id]?.metric_kind ?? 'Not defined',
        },
        draggable: false,
      };
    });

    const scalingComponentNodeXOffset =
      (maxWidth - scalingComponents?.length * POSITION_X_OFFSET) / 2;
    const scalingComponentNodes = scalingComponents.map(
      (scalingComponent: ScalingComponentUI, index) => {
        return {
          // Default properties
          id: scalingComponent.id,
          type: 'scalingComponent',
          position: {
            x: POSITION_X_OFFSET * index + scalingComponentNodeXOffset,
            y: POSITION_Y_OFFSET * 2,
          },
          data: {
            label: scalingComponent.id,
            componentKind:
              scalingComponentMap[scalingComponent.id]?.component_kind ??
              'Not defined',
          },
          draggable: false,
        };
      }
    );

    return [...planNodes, ...metricNodes, ...scalingComponentNodes];
  }, [plans, metricMap, scalingComponentMap]);

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
      nodeTypes={nodeTypes}
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
