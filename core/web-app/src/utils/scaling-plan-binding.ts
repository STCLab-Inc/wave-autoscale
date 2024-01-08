import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

export function generateScalingPlanDefinition({
  kind,
  id,
  db_id,
  metadata,
  plans,
  enabled,
}: {
  kind: string;
  id?: string;
  db_id?: string;
  metadata?: any;
  plans?: any[];
  enabled?: boolean;
}) {
  return {
    kind: kind,
    id,
    db_id,
    metadata: metadata ?? {},
    plans: plans ?? [],
    enabled,
  } as ScalingPlanDefinition;
}
