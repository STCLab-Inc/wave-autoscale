'use client';

import ScalingComponentService from '@/services/scaling-component';
import { getScalingComponentPlanKeyTypes } from '@/utils/scaling-component-binding';
import { set } from 'lodash';
import { useEffect, useMemo, useState } from 'react';
import { Controller, useWatch } from 'react-hook-form';
import { getMetadataFormControls } from '../../metadata-form-controls';

// Metric Types
const componentPlanKeyTypes = getScalingComponentPlanKeyTypes();

export default function ScalingComponentPlanSelect(props: any) {
  const { control, index, field } = props;
  const value = useWatch({
    name: 'scaling_components',
    control,
  });

  console.log({ value });
  const [scalingComponentOptions, setScalingComponentOptions] = useState<any[]>(
    []
  );

  // Fetch Scaling Components
  useEffect(() => {
    const fetch = async () => {
      const scalingComponents =
        (await ScalingComponentService.getScalingComponents()) ?? [];
      console.log({ scalingComponents });
      const newScalingComponentOptions = scalingComponents.map(
        (scalingComponent: any) => {
          return (
            <option
              key={scalingComponent.id}
              value={JSON.stringify({
                id: scalingComponent.id,
                component_kind: scalingComponent.component_kind,
              })}
            >{`[${scalingComponent.component_kind}] ${scalingComponent.id}`}</option>
          );
        }
      );
      setScalingComponentOptions(newScalingComponentOptions);
    };
    fetch();
  }, []);

  const selectedComponent = value[index]?.component;
  // console.log({ selectedComponentId });

  const metadataFormControls = useMemo(() => {
    if (selectedComponent) {
      const { component_kind } = JSON.parse(selectedComponent);
      const keyTypes = componentPlanKeyTypes.find(
        (componentKeyType) => componentKeyType.componentName === component_kind
      )?.keyTypes;
      if (keyTypes) {
        return keyTypes.map((keyType) => {
          return (
            <div className="form-control mb-1 w-full" key={keyType.key}>
              <label className="input-group-sm input-group input-group-vertical">
                <span>{keyType.key}</span>
                <Controller
                  control={control}
                  name={`scaling_components.${index}.${keyType.key}`}
                  render={({ field }) => (
                    <input
                      className="input-bordered input"
                      type={keyType.type === 'number' ? 'number' : 'text'}
                      {...field}
                    />
                  )}
                />
              </label>
            </div>
          );
        });
      }
    }
  }, [selectedComponent]);

  return (
    <div key={`${field.id}`}>
      <div className="form-control mb-1 w-full">
        <label className="input-group-sm input-group input-group-vertical">
          <span>Scaling Component</span>
          <Controller
            control={control}
            name={`scaling_components.${index}.component`}
            render={({ field }) => (
              <select className="select-bordered select" defaultValue="" {...field}>
                <option value="" disabled>Pick one</option>
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
