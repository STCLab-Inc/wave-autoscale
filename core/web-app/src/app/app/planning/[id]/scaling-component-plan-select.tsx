'use client';

import ScalingComponentService from '@/services/scaling-component';
import { getScalingComponentPlanKeyTypes } from '@/utils/scaling-component-binding';
import { MouseEventHandler, useEffect, useMemo, useState } from 'react';
import { Controller, useWatch } from 'react-hook-form';

// Metric Types
const componentPlanKeyTypes = getScalingComponentPlanKeyTypes();

export default function ScalingComponentPlanSelect(props: any) {
  // react-hook-form useFieldArray props
  const { control, index, field, remove } = props;
  const value = useWatch({
    name: 'scaling_components',
    control,
  });
  // The selected scaling component
  const selectedComponentId = value?.[index]?.component_id;
  const existingComponentIds = value?.map((item: any) => item.component_id);

  // Fetch Scaling Components from API that the user created
  const [myScalingComponents, setMyScalingComponents] = useState<any[]>([]);
  // For <select> options
  const scalingComponentOptions = useMemo(() => {
    return myScalingComponents.reduce((options: any, scalingComponent: any) => {
      if (
        scalingComponent.id === selectedComponentId ||
        !existingComponentIds?.includes(scalingComponent.id)
      ) {
        options.push(
          <option
            key={scalingComponent.id}
            value={scalingComponent.id}
          >{`[${scalingComponent.component_kind}] ${scalingComponent.id}`}</option>
        );
      }
      return options;
    }, []);
  }, [myScalingComponents, existingComponentIds]);

  // For the scaling component map that maps component id to component kind
  const scalingComponentMap = useMemo(() => {
    return myScalingComponents.reduce((map: any, scalingComponent: any) => {
      map[scalingComponent.id] = scalingComponent.component_kind;
      return map;
    }, {});
  }, [myScalingComponents]);

  // Fetch Scaling Components from API that the user created
  useEffect(() => {
    const fetch = async () => {
      const scalingComponents =
        (await ScalingComponentService.getScalingComponents()) ?? [];

      setMyScalingComponents(scalingComponents);
    };
    fetch();
  }, []);

  const metadataFormControls = useMemo(() => {
    console.log('selectedComponentId', selectedComponentId);
    if (!selectedComponentId) {
      return null;
    }
    const componentKind = scalingComponentMap[selectedComponentId];
    console.log('componentKind', componentKind);
    if (!componentKind) {
      return null;
    }
    const keyTypes = componentPlanKeyTypes.find(
      (componentKeyType) => componentKeyType.componentName === componentKind
    )?.keyTypes;
    if (keyTypes) {
      return keyTypes.map((keyType) => {
        return (
          <div className="form-control mb-1 w-full" key={keyType.key}>
            <label className="input-group-sm input-group">
              <span className="input-group-label-fixed">{keyType.key}</span>
              <Controller
                control={control}
                name={`scaling_components.${index}.${keyType.key}`}
                render={({ field }) => (
                  <input
                    className="input-bordered input input-sm flex-1"
                    // The value coule be a expression of JavaScript
                    type="text"
                    // type={keyType.type === 'number' ? 'number' : 'text'}
                    {...field}
                  />
                )}
              />
            </label>
          </div>
        );
      });
    }
  }, [selectedComponentId, scalingComponentMap]);

  const onRemove: MouseEventHandler<HTMLButtonElement> = (event) => {
    event.preventDefault();
    if (!confirm('Are you sure you want to remove this scaling component?')) {
      return;
    }
    remove(index);
  };

  return (
    <div key={`${field.id}`} className="mb-5 border-b border-base-300 pb-2">
      <div className="mb-2 flex flex-row justify-end">
        <button className="btn-error btn-outline btn-xs btn" onClick={onRemove}>
          Remove
        </button>
      </div>
      <div className="form-control mb-2 w-full">
        <label className="input-group-sm input-group">
          <span className="input-group-label-fixed">Component</span>
          <Controller
            control={control}
            name={`scaling_components.${index}.component_id`}
            render={({ field }) => (
              <select
                className="select-bordered select select-sm flex-1"
                defaultValue=""
                {...field}
              >
                <option value="" disabled>
                  Pick one
                </option>
                {scalingComponentOptions}
              </select>
            )}
          />
        </label>
      </div>
      {metadataFormControls}
    </div>
  );
}
