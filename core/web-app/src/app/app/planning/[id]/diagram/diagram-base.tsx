'use client';

import 'reactflow/dist/style.css';
import { usePlanStore } from '../plan-store';
import PlanningDiagramFlow from './diagram-flow';
import { ReactFlowProvider } from 'reactflow';

export default function PlanningDetailDiagramBase() {
  const addPlan = usePlanStore((state) => state.addPlan);

  // Events
  const onClickAddNode = () => {
    console.log('onClickAddNode');
    addPlan();
  };

  return (
    <>
      <ReactFlowProvider>
        <PlanningDiagramFlow />
      </ReactFlowProvider>
      <button
        className="btn-primary btn-sm btn absolute left-8 top-4"
        onClick={onClickAddNode}
      >
        Add Node
      </button>
    </>
  );
}
