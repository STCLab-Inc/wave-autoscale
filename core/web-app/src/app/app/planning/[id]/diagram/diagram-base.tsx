'use client';

import 'reactflow/dist/style.css';
import { usePlanStore } from '../../plan-store';
import PlanningDiagramFlow from './diagram-flow';
import { ReactFlowProvider } from 'reactflow';
import { useParams } from 'next/navigation';

export default function PlanningDetailDiagramBase() {
  const addPlanItem = usePlanStore((state) => state.addPlanItem);
  // Events
  const onClickAddPlan = () => {
    addPlanItem();
  };

  return (
    <>
      <ReactFlowProvider>
        <PlanningDiagramFlow />
      </ReactFlowProvider>
      <button
        className="absolute left-4 top-4 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
        onClick={onClickAddPlan}
      >
        ADD PLAN
      </button>
    </>
  );
}
