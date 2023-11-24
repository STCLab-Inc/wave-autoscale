import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

export function generateScalingPlanDefinition({
  kind,
  id,
  db_id,
  metadata,
  plans,
}: {
  kind: string;
  id?: string;
  db_id?: string;
  metadata?: any;
  plans?: any[];
}) {
  return {
    kind: kind,
    id,
    db_id,
    metadata,
    plans,
  } as ScalingPlanDefinition;
}
