'use client';

import React, { useMemo, useEffect, useState } from 'react';
import ReactFlow, {
  Background,
  BackgroundVariant,
  Controls,
  Edge,
  NodeChange,
  MarkerType,
} from 'reactflow';
import { flatten, keyBy, unionBy } from 'lodash';
import { produce } from 'immer';
import * as acorn from 'acorn';
import * as walk from 'acorn-walk';

import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import MetricService from '@/services/metric';
import ScalingComponentService from '@/services/scaling-component';

import DiagramNodePlan from './diagram-node-plan';
import DiagramNodeMetric from './diagram-node-metric';
import DiagramNodeComponent from './diagram-node-component';

const nodeTypes = {
  scalingPlan: DiagramNodePlan,
  metric: DiagramNodeMetric,
  scalingComponent: DiagramNodeComponent,
};

export interface MetricUI {
  id: string;
  selected?: boolean;
}
export interface ScalingComponentUI {
  id: string;
  selected?: boolean;
}

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function DiagramFlow({
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
  const [metricMap, setMetricMap] = useState<any>({});
  const [scalingComponentMap, setScalingComponentMap] = useState<any>({});

  const [renderingScalingPlansItem, setRenderingScalingPlansItem] = useState<
    ScalingPlanDefinitionEx | undefined
  >(undefined);

  useEffect(() => {
    const fetch = async () => {
      const promises = [
        MetricService.getMetrics(),
        ScalingComponentService.getScalingComponents(),
      ];
      const results = await Promise.all(promises);
      setMetricMap(keyBy(results[0], 'id'));
      setScalingComponentMap(keyBy(results[1], 'id'));
      if (scalingPlansItem) {
        const newRenderingPlansItem = produce(scalingPlansItem, (draft) => {
          draft.plans.forEach((plan) => {
            const expression = plan.expression;
            const metricIdsInScalingPlan = new Set<string>();
            const scalingComponentIdsInScalingPlan = new Set<string>();
            if (expression) {
              // 1. Metric Ids
              try {
                const ast = acorn.parse(expression, {
                  ecmaVersion: 2020,
                  sourceType: 'script',
                });
                walk.simple(ast, {
                  ObjectExpression(node: any) {
                    const metricIdProperty = node.properties.find(
                      (property: any) => {
                        return (
                          property.key.type === 'Identifier' &&
                          property.key.name === 'metric_id'
                        );
                      }
                    );

                    if (
                      metricIdProperty &&
                      metricIdProperty.value.type === 'Literal'
                    ) {
                      const metricIdValue = metricIdProperty.value.value;
                      metricIdsInScalingPlan.add(metricIdValue);
                    }
                  },
                });
              } catch (error) {
                metricIdsInScalingPlan.add('Not found');
                // TODO: Error handling
              }
            }

            // 2. Scaling Component Ids
            plan.scaling_components?.forEach((component) => {
              scalingComponentIdsInScalingPlan.add(component.component_id);
            });

            // Update the plan with the new ui property
            plan.ui = {
              ...(plan.ui ?? {}),
              metrics: Array.from(metricIdsInScalingPlan).map(
                (metricId) =>
                  ({
                    id: metricId,
                    selected: false,
                  } as MetricUI)
              ),
              scalingComponents: Array.from(
                scalingComponentIdsInScalingPlan
              ).map(
                (scalingComponentId) =>
                  ({
                    id: scalingComponentId,
                    selected: false,
                  } as ScalingComponentUI)
              ),
            };
          });
        });

        // Update the state with the new renderingPlansItem
        setRenderingScalingPlansItem(newRenderingPlansItem);
      }
    };

    fetch();
  }, [scalingPlansItem]);

  // Nodes are the plans and metrics
  const nodes = useMemo(() => {
    if (renderingScalingPlansItem) {
      // Use renderingPlansItem instead of plansItem for rendering
      const POSITION_X_OFFSET = 400;
      const POSITION_Y_OFFSET = 200;
      const metrics = unionBy(
        flatten(
          renderingScalingPlansItem.plans?.map((plan) => plan?.ui?.metrics)
        ),
        'id'
      );
      const scalingComponents = unionBy(
        flatten(
          renderingScalingPlansItem.plans?.map(
            (plan) => plan?.ui?.scalingComponents
          )
        ),
        'id'
      );

      const maxWidth =
        Math.max(
          renderingScalingPlansItem.plans?.length,
          metrics?.length,
          scalingComponents?.length
        ) * POSITION_X_OFFSET;

      const metricNodes = metrics.map((metric: MetricUI, index) => {
        return {
          // Default properties
          id: metric?.id,
          type: 'metric',
          position: {
            x: 0,
            y:
              (POSITION_Y_OFFSET * renderingScalingPlansItem.plans?.length) /
                2 -
              POSITION_Y_OFFSET / 2,
          },
          data: {
            label: metric?.id,
            collector: metricMap[metric?.id]?.collector,
          },
          draggable: false,
        };
      });

      const scalingPlanNodes = renderingScalingPlansItem.plans.map(
        (plan, index) => {
          return {
            // Default properties
            id: plan.id,
            type: 'plan',
            position: {
              x: 2 * POSITION_X_OFFSET,
              y: index * POSITION_Y_OFFSET,
            },
            data: { label: plan.id },
            draggable: false,
            // Get the ui property from the plan
            ...plan?.ui,
          };
        }
      );

      const scalingComponentNodes = scalingComponents.map(
        (scalingComponent: ScalingComponentUI, index) => {
          return {
            // Default properties
            id: scalingComponent?.id,
            type: 'scalingComponent',
            position: {
              x: 4 * POSITION_X_OFFSET,
              y:
                (POSITION_Y_OFFSET * renderingScalingPlansItem.plans?.length) /
                  2 -
                POSITION_Y_OFFSET / 2,
            },
            data: {
              label: scalingComponent?.id,
              componentKind:
                scalingComponentMap[scalingComponent?.id]?.component_kind ??
                'Not defined',
            },
            draggable: false,
          };
        }
      );

      return [...scalingPlanNodes, ...metricNodes, ...scalingComponentNodes];
    }
  }, [renderingScalingPlansItem, metricMap, scalingComponentMap]);

  const edges = useMemo(() => {
    if (renderingScalingPlansItem) {
      // Use renderingPlansItem instead of plansItem for rendering
      const edgesForScalingPlanAndMetric: Edge[] = [];
      const edgesForScalingPlanAndScalingComponent: Edge[] = [];
      renderingScalingPlansItem.plans.forEach((plan) => {
        const metrics = plan?.ui?.metrics;
        if (metrics?.length) {
          metrics?.forEach((metric: MetricUI) => {
            edgesForScalingPlanAndMetric.push({
              id: `${plan.id}-${metric.id}`,
              source: metric.id,
              target: plan.id,
              animated: true,
              type: 'default',
              markerEnd: {
                type: MarkerType.ArrowClosed,
                width: 20,
                height: 20,
              },
            });
          });
        }

        const scalingComponents = plan?.ui?.scalingComponents;
        if (scalingComponents?.length) {
          scalingComponents?.forEach((scalingComponent: ScalingComponentUI) => {
            edgesForScalingPlanAndMetric.push({
              id: `${plan.id}-${scalingComponent?.id}`,
              source: plan.id,
              target: scalingComponent?.id,
              animated: true,
              type: 'default',
              markerEnd: {
                type: MarkerType.ArrowClosed,
                width: 20,
                height: 20,
              },
            });
          });
        }
      });
      return [
        ...edgesForScalingPlanAndMetric,
        ...edgesForScalingPlanAndScalingComponent,
      ];
    }
  }, [renderingScalingPlansItem, metricMap, scalingComponentMap]);

  const onNodesChange = (nodes: NodeChange[]) => {
    if (scalingPlansItem) {
      // Find the plan by id
      const findPlan = (planId: string) =>
        scalingPlansItem.plans.find((plan) => plan.id === planId);

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
            /* updatePlanItemUI(newPlan); */
          }
        } else if (type === 'select') {
          // Update the plan with the new selected state
          const plan = findPlan(node.id);
          if (plan) {
            const newPlan = produce(plan, (draft) => {
              draft.ui = { ...draft.ui, selected: node.selected };
            });
            setDetailsModalFlag(true);
            /* updatePlanItemUI(newPlan); */
          }
        }
      });
    }
  };

  return (
    <ReactFlow
      nodes={nodes}
      edges={edges}
      // Reference: https://reactflow.dev/learn/troubleshooting
      // Description: It looks like you have created a new nodeTypes or edgeTypes object. If this wasn't on purpose please define the nodeTypes/edgeTypes outside of the component or memoize them.
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
