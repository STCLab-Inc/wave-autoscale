'use client';

import { useParams } from 'next/navigation';
import { usePlanStore } from '../plan-store';

export default function PlanningDetailControls() {
  const push = usePlanStore((state) => state.push);

  const onClickDeploy = async () => {
    await push();
    alert('Deployed!');
  };

  return (
    <div className="flex items-center">
      <button
        className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
        onClick={onClickDeploy}
      >
        DEPLOY
      </button>
    </div>
  );
}
