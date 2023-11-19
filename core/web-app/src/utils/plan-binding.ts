import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { nanoid } from 'nanoid';

export function generatePlanDefinition({
  id = '',
  db_id = '',
  title,
  plans = [],
}: {
  id?: string;
  db_id?: string;
  title: string;
  plans?: any[];
}) {
  if (!id) {
    id = `generated-id-${nanoid(5)}`;
  }
  return {
    kind: 'ScalingPlan',
    db_id,
    id,
    metadata: { title },
    plans,
  } as ScalingPlanDefinition;
}
