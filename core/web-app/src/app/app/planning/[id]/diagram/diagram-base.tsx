'use client';

import 'reactflow/dist/style.css';
import { usePlanStore } from '../plan-store';
import PlanningDiagramFlow from './diagram-flow';
import { ReactFlowProvider } from 'reactflow';
import PlanDetailDrawer from '@/app/app/planning/[id]/plan-drawer';

export default function PlanningDetailDiagramBase() {
  const addPlan = usePlanStore((state) => state.addPlan);
  // Events
  const onClickAddPlan = () => {
    addPlan();
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
