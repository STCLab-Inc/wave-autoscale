'use client';

import PlanService from '@/services/plan';
import { generatePlanDefinition } from '@/utils/plan-binding';
import classNames from 'classnames';
import { useState } from 'react';
import { useForm } from 'react-hook-form';

export default function PlanningSidebar() {
  const [addModalToggle, setAddModalToggle] = useState(false);
  const { register, reset, setFocus, handleSubmit } = useForm();

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
    const plan = generatePlanDefinition({ title });
    try {
      const response = await PlanService.createPlan(plan);
      setAddModalToggle(false);
    } catch (e) {}
  };
  const onClickCancelInModal = () => {
    setAddModalToggle(false);
  };

  return (
    <>
      <aside className="flex w-80 flex-col border-r border-base-300 bg-base-200">
        {/* Header of aside */}
        <div className="prose flex h-14 w-full items-center justify-start border-b border-base-300 px-3">
          <div className="flex flex-1 flex-row items-center justify-start">
            <h4 className="m-0">Plans</h4>
            <span className="badge ml-2">4</span>
          </div>
          <button className="btn-primary btn-sm btn" onClick={onClickAdd}>
            Add
          </button>
        </div>
        {/* Plan List */}
        <div className="flex-1 overflow-y-auto">{/* Plan Item */}</div>
      </aside>
      <div className={classNames('modal', { 'modal-open': addModalToggle })}>
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
