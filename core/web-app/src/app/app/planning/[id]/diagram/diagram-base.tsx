'use client';

import 'reactflow/dist/style.css';
import { usePlanStore } from '../../plan-store';
import PlanningDiagramFlow from './diagram-flow';
import { ReactFlowProvider } from 'reactflow';
import { useParams } from 'next/navigation';

export default function PlanningDetailDiagramBase() {
  const { id: scalingPlanId } = useParams();
  const addPlanItem = usePlanStore((state) => state.addPlanItem);
  // Events
  const onClickAddPlan = () => {
    addPlanItem(scalingPlanId);
  };

  return (
    <>
      <ReactFlowProvider>
        <PlanningDiagramFlow />
      </ReactFlowProvider>
      <button
        className="btn-primary btn-sm btn absolute left-8 top-4"
        onClick={onClickAddPlan}
      >
        Add Plan
      </button>
    </>
  );
}
