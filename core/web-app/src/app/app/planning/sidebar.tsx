'use client';

import PlanService from '@/services/plan';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { generatePlanDefinition } from '@/utils/plan-binding';
import classNames from 'classnames';
import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { usePlanStore } from './plan-store';

export default function PlanningSidebar() {
  const [addModalToggle, setAddModalToggle] = useState(false);
  const needToSave = usePlanStore((state) => state.needToSave);
  const currentScalingPlanState = usePlanStore(
    (state) => state.currentScalingPlanState
  );
  console.log({ currentScalingPlanState });
  // Used to force refresh
  const [timestamp, setTimestamp] = useState(Date.now());
  const { register, reset, setFocus, handleSubmit } = useForm();
  const [plans, setPlans] = useState([]);
  const params = useParams();
  const selectePlanId = params?.id;

  useEffect(() => {
    const fetchPlans = async () => {
      const response = await PlanService.getPlans();
      setPlans(response);
    };
    fetchPlans();
  }, [timestamp]);

  const onClickAdd = () => {
    reset();
    setAddModalToggle(true);
    // TODO: Fix this hack
    setTimeout(() => {
      setFocus('title');
    }, 100);
  };

  const onSubmitAddDialog = async (data: any) => {
    const { title } = data;
    if (!title?.trim()) {
      return;
    }
    const plan = generatePlanDefinition({ title });
    try {
      const response = await PlanService.createPlan(plan);
      setTimestamp(Date.now());
      setAddModalToggle(false);
    } catch (e) {}
  };
  const onClickCancelInModal = (event: any) => {
    // Check if this event is from the element itself
    if (event.target !== event.currentTarget) {
      return;
    }
    setAddModalToggle(false);
  };

  return (
    <>
      <aside className="flex w-80 flex-col border-r border-base-300 bg-base-200">
        {/* Header of aside */}
        <div className="prose flex h-14 w-full items-center justify-start border-b border-base-300 px-3">
          <div className="flex flex-1 flex-row items-center justify-start">
            <h4 className="m-0">Plans</h4>
            <span className="badge ml-2">
              {plans !== undefined ? plans.length : undefined}
            </span>
          </div>
          <button className="btn-primary btn-sm btn" onClick={onClickAdd}>
            Add
          </button>
        </div>
        {/* Plan List */}
        <div className="flex-1 overflow-y-auto p-3">
          {/* Plan Item */}
          {plans?.map((plan: ScalingPlanDefinition) => (
            <Link
              href={`/app/planning/${plan.db_id}`}
              key={plan.db_id}
              className={classNames(
                'mb mb-2 flex h-8 w-full cursor-pointer items-center justify-start rounded-xl px-3',
                {
                  'bg-primary': plan.db_id === selectePlanId,
                }
              )}
            >
              <div
                className={classNames('prose', {
                  'text-white': plan.db_id === selectePlanId,
                })}
              >
                {plan.kind}
                {currentScalingPlanState && needToSave(plan.db_id) && '*'}
              </div>
            </Link>
          ))}
        </div>
      </aside>
      {/* Add a Plan Modal */}
      <div
        className={classNames('modal', { 'modal-open': addModalToggle })}
        onClick={onClickCancelInModal}
      >
        <div className="modal-box">
          <form onSubmit={handleSubmit(onSubmitAddDialog)}>
            <h3 className="text-lg font-bold">Add a Plan</h3>
            <div className="form-control w-full">
              <label className="label">
                <span className="label-text">Plan Title</span>
              </label>
              <input
                {...register('title')}
                type="text"
                placeholder="Title"
                className="input-bordered input w-full"
              />
            </div>

            <div className="modal-action">
              <button className="btn-ghost btn" onClick={onClickCancelInModal}>
                Cancel
              </button>
              <button className="btn-primary btn" type="submit">
                Add
              </button>
            </div>
          </form>
        </div>
      </div>
    </>
  );
}
