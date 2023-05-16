'use client';

import { useParams } from 'next/navigation';
import { usePlanStore } from '../plan-store';

export default function PlanningDetailControls() {
  const { id: scalingPlanId } = useParams();
  const push = usePlanStore((state) => state.push);

  const onClickDeploy = async () => {
    await push(scalingPlanId);
    alert('deployed!');
  };

  return (
    <div className="flex items-center pr-3">
      <button className="btn-primary btn-sm btn" onClick={onClickDeploy}>
        Deploy
      </button>
    </div>
  );
}
