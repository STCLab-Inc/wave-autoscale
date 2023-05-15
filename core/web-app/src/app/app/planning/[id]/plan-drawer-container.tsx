'use client';

import dynamic from 'next/dynamic';
import PlanDetailDrawer from './plan-drawer';
import { usePlanStore } from './plan-store';

/**
 * This is a container component that renders the PlanDetailDrawer component in the client side.
 */

// This is a dynamic import that will only be rendered in the client side.
// This is because AceEditor is not compatible with SSR.
const PlanDrawerDynamic = dynamic(() => import('./plan-drawer'), {
  ssr: false,
});

export default function PlanDrawerContainer() {
  const selectedPlan = usePlanStore((state) => state.selectedPlan);

  return <>{selectedPlan ? <PlanDrawerDynamic /> : undefined}</>;
}
