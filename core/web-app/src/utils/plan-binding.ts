import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { nanoid } from 'nanoid';

export function generatePlanDefinition({
  id = `generated-id-${nanoid(5)}`,
  db_id = '',
  title,
  plans = [],
}: {
  id?: string;
  db_id?: string;
  title: string;
  plans?: any[];
}) {
  return {
    kind: 'ScalingPlan',
    db_id,
    id,
    metadata: { title },
    plans,
  } as ScalingPlanDefinition;
}
