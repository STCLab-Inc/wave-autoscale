'use client';

import dynamic from 'next/dynamic';
import { usePlanStore } from '../plan-store';

/**
 * This is a container component that renders the PlanDetailDrawer component in the client side.
 */

// This is a dynamic import that will only be rendered in the client side.
// This is because AceEditor is not compatible with SSR.
const PlanItemDrawerDynamic = dynamic(() => import('./plan-item-drawer'), {
  ssr: false,
});

export default function PlanItemDrawerContainer() {
  // Only render the PlanItemDrawerDynamic component if there is a selected plan.
  const selectedPlan = usePlanStore(
    (state) => state.currentScalingPlanState?.selectedPlan
  );

  return (
    <>
      {selectedPlan ? (
        <PlanItemDrawerDynamic planItemDefinition={selectedPlan} />
      ) : undefined}
    </>
  );
}
