import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import JSYaml from 'js-yaml';

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
