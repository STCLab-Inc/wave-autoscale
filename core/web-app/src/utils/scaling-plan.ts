import { MetricDefinition } from '@/types/bindings/metric-definition';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { GetJSParamDefinition } from '@/types/get-js-param-definition';
import * as acorn from 'acorn';
import * as walk from 'acorn-walk';
import JSYaml from 'js-yaml';
import { forEach } from 'lodash';

export function generateScalingPlanDefinition({
  kind,
  id,
  db_id,
  metadata,
  variables,
  plans,
  enabled,
}: {
  kind: string;
  id?: string;
  db_id?: string;
  metadata?: any;
  variables?: any;
  plans?: any[];
  enabled?: boolean;
}) {
  return {
    kind: kind,
    id,
    db_id,
    metadata: metadata ?? {},
    variables: variables ?? {},
    plans: plans ?? [],
    enabled,
  } as ScalingPlanDefinition;
}

export function serializeScalingPlanDefinition(
  scalingPlanDefinition: ScalingPlanDefinition
) {
  const { kind, id, metadata, plans, enabled, variables } =
    scalingPlanDefinition;
  const editedPlans = plans?.map((planItem) => {
    const { ui, ...rest } = planItem;
    return {
      ...rest,
    };
  });
  const serialized = JSYaml.dump({
    kind,
    id,
    metadata,
    variables,
    plans: editedPlans,
    enabled,
  });
  return serialized;
}

export function serializeScalingPlanDefinitions(
  scalingPlanDefinitions: ScalingPlanDefinition[]
) {
  const serialized = scalingPlanDefinitions.map((scalingPlanDefinition) =>
    serializeScalingPlanDefinition(scalingPlanDefinition)
  );
  const result = serialized.join('\n---\n');
  return result;
}

export function deserializeScalingPlanDefinition(serialized: string) {
  let deserialized: ScalingPlanDefinition = JSYaml.load(
    serialized
  ) as ScalingPlanDefinition;
  return deserialized;
}

export function extractMetricsFromScalingPlan(
  scalingPlan: ScalingPlanDefinition
) {
  const metrics: GetJSParamDefinition[] = [];

  scalingPlan?.plans?.forEach((plan) => {
    const expression = plan.expression;
    if (expression) {
      const metricDefinition = extractGetJsParamsFromExpression(expression);
      if (metricDefinition) {
        metrics.push(metricDefinition);
      }
    }
  });

  forEach(scalingPlan?.variables, (value, key) => {
    if (typeof value === 'string') {
      const metricDefinition = extractGetJsParamsFromExpression(value);
      if (metricDefinition) {
        metrics.push(metricDefinition);
      }
    }
  });

  return metrics;
}

function extractGetJsParamsFromExpression(expression: string) {
  let metricDefinition: GetJSParamDefinition | undefined;
  try {
    const ast = acorn.parse(expression, {
      ecmaVersion: 2020,
      sourceType: 'script',
    });
    walk.simple(ast, {
      ObjectExpression(node: any) {
        const metricIdProperty = node.properties.find((property: any) => {
          return (
            property.key.type === 'Identifier' &&
            property.key.name === 'metric_id' &&
            property.value.type === 'Literal'
          );
        });
        if (!metricIdProperty) {
          return;
        }
        metricDefinition = {
          metricId: metricIdProperty.value.value,
        };

        const nameProperty = node.properties.find((property: any) => {
          return (
            property.key.type === 'Identifier' && property.key.name === 'name'
          );
        });
        if (nameProperty) {
          metricDefinition.name = nameProperty.value.value;
        }

        const tagsProperty = node.properties.find((property: any) => {
          return (
            property.key.type === 'Identifier' && property.key.name === 'tags'
          );
        });
        if (tagsProperty) {
          tagsProperty.value.properties.forEach((property: any) => {
            if (!metricDefinition) {
              return;
            }
            if (!metricDefinition?.tags) {
              metricDefinition.tags = {};
            }
            metricDefinition.tags[property.key.name] = property.value.raw;
          });
        }

        console.log('metricDefinition', metricDefinition);

        return metricDefinition;
      },
    });
  } catch (error) {
    // TODO: Error handling
    console.error(error);
  }
  return metricDefinition;
}
