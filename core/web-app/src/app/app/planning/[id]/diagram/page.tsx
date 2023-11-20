'use client';

import { useParams } from 'next/navigation';
import PlanningDetailDiagramBase from './diagram-base';
import { useEffect } from 'react';
import { usePlanStore } from '../../plan-store';

export default function PlanningDetailDiagramPage() {
  const { id: scalingPlanId } = useParams();
  const load = usePlanStore((state) => state.load);

  // If scalingPlanId changes, fetch the scaling plan then it updates plans and nodes.
  useEffect(() => {
    const fetch = async (id: string) => {
      await load(id);
    };

    if (Array.isArray(scalingPlanId)) {
      fetch(scalingPlanId[0]);
    } else {
      fetch(scalingPlanId);
    }
  }, [scalingPlanId]);

  return (
    <div className="h-full w-full">
      <PlanningDetailDiagramBase />
    </div>
  );
}
