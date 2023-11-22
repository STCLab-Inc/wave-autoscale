import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { nanoid } from 'nanoid';

export function generatePlanDefinition({
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
