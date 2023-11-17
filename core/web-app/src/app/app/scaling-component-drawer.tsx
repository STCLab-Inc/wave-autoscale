'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { forEach } from 'lodash';

import MetricService from '@/services/metric';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';
import {
  generateScalingComponentDefinition,
  getScalingComponentKeyTypes,
} from '@/utils/scaling-component-binding';
import ScalingComponentService from '@/services/scaling-component';
import { getMetadataFormControls } from './metadata-form-controls';

// Metric Types
const componentKeyTypes = getScalingComponentKeyTypes();
const componentOptions = componentKeyTypes.map((componentKeyType) => (
  <option
    key={componentKeyType.componentName}
    value={componentKeyType.componentName}
  >
    {componentKeyType.componentName}
  </option>
));

export default function ScalingComponentDetailDrawer({
  componentDefinition,
}: {
  componentDefinition?: ScalingComponentDefinition;
}) {
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    getValues,
    reset,
    formState: { errors },
  } = useForm();
  const router = useRouter();
  const dbId = componentDefinition?.db_id;
  const isNew = !dbId;
  const [selectedComponentKind, setSelectedComponentKind] = useState<string>();

  const metadataFormControls = useMemo(() => {
    if (selectedComponentKind) {
      const keyTypes = componentKeyTypes.find(
        (componentKeyType) =>
          componentKeyType.componentName === selectedComponentKind
      )?.keyTypes;
      if (keyTypes) {
        return getMetadataFormControls(keyTypes, register);
      }
    }
  }, [selectedComponentKind]);

  const goBack = (refresh?: boolean) => {
    let path = window.location.href;
    path = path.slice(0, path.lastIndexOf('/'));
    router.push(path);
    if (refresh) {
      router.refresh();
    }
  };

  //
  // Events
  //

  // Set default values
  useEffect(() => {
    if (isNew) {
      return;
    }
    const { id, component_kind, metadata, ...rest } = componentDefinition;
    setValue('id', id);
    setValue('component_kind', component_kind);
    forEach(metadata, (value, key) => {
      setValue(key, value);
    });
    setSelectedComponentKind(component_kind);
  }, [componentDefinition, isNew]);

  const onClickOverlay = () => {
    goBack(false);
  };

  const onChangeComponentKind = (e: any) => {
    const componentKind = e.target.value;
    setSelectedComponentKind(componentKind);

    // Clear metadata
    const id = getValues('id');
    reset();
    setValue('id', id);
    setValue('component_kind', componentKind);
  };

  const onClickRemove = async () => {
    if (isNew || !dbId) {
      return;
    }
    try {
      await ScalingComponentService.deleteScalingComponent(dbId);
      goBack(true);
    } catch (error) {
      console.log(error);
    }
  };

  const onSubmit = async (data: any) => {
    const { id, component_kind, ...metadata } = data;
    const componentDefinition = generateScalingComponentDefinition({
      id,
      db_id: isNew ? '' : dbId,
      component_kind,
      metadata,
    });
    try {
      if (isNew) {
        // const result = await MetricService.createMetric(componentDefinition);
        const result = await ScalingComponentService.createScalingComponent(
          componentDefinition
        );
        console.log({ result, isNew });
      } else {
        // const result = await MetricService.updateMetric(componentDefinition);
        const result = await ScalingComponentService.updateScalingComponent(
          componentDefinition
        );
        console.log({ result });
      }
      goBack(true);
    } catch (error) {
      console.log(error);
    }
  };

  return (
    <div className="drawer drawer-end fixed inset-y-0 bottom-0 top-16 z-50">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
        />
        <div className="drawer-content min-w-[32rem] overflow-y-auto bg-base-100 p-4">
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="mb-4 flex items-center justify-between">
              <h2 className="font-bold">Scaling Component</h2>
              <div>
                {isNew ? undefined : (
                  <button
                    type="button"
                    className="btn-error btn-sm btn mr-2"
                    onClick={onClickRemove}
                  >
                    Remove
                  </button>
                )}

                <button type="submit" className="btn-primary btn-sm btn">
                  Save
                </button>
              </div>
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">Scaling Component Type</span>
                <span className="label-text-alt"></span>
              </label>
              <select
                className="select-bordered select"
                defaultValue="Scaling Component Kind"
                {...register('component_kind', {
                  required: true,
                  onChange: onChangeComponentKind,
                })}
              >
                <option disabled>Scaling Component Kind</option>
                {componentOptions}
              </select>
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">ID</span>
                <span className="label-text-alt">used as a variable name</span>
              </label>
              <input
                type="text"
                placeholder="Type here"
                className="input-bordered input w-full"
                {...register('id', { required: true })}
              />
            </div>

            {metadataFormControls}
          </form>
        </div>
      </div>
    </div>
  );
}
