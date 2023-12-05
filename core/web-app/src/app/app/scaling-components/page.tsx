'use client';

import React, { useEffect, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';

import ScalingComponentService from '@/services/scaling-component';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

import ScalingComponentDetailDrawer from './scaling-component-drawer';
import ContentHeader from '../common/content-header';
import { TableComponent } from './scaling-component-table';

async function getScalingComponents() {
  const scalingComponents =
    await ScalingComponentService.getScalingComponents();
  return scalingComponents;
}

interface ScalingComponentDefinitionEx extends ScalingComponentDefinition {
  isChecked: boolean;
}

export default function ScalingComponentsPage() {
  const router = useRouter();
  const searchParams = useSearchParams();

  const pageParam = searchParams.get('page');
  const page = pageParam ? parseInt(pageParam, 10) : 1;
  const sizeParam = searchParams.get('size');
  const size = sizeParam ? parseInt(sizeParam, 10) : 10;

  const [scalingComponents, setScalingComponents] = useState<
    ScalingComponentDefinitionEx[]
  >([]);
  const [scalingComponentsItem, setScalingComponentsItem] =
    useState<ScalingComponentDefinitionEx>();

  const fetchScalingComponents = async () => {
    try {
      const id = scalingComponentsItem?.db_id;
      let scalingComponentsData = await getScalingComponents();
      setScalingComponents(scalingComponentsData);
      setScalingComponentsItem(
        scalingComponentsData.find(
          (item: ScalingComponentDefinitionEx) => item.db_id === id
        )
      );
    } catch (error) {
      console.error(error);
      return [];
    }
  };

  useEffect(() => {
    fetchScalingComponents();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const [selectAll, setSelectAll] = useState(false);

  const handleSelectAll = (event: React.ChangeEvent<HTMLInputElement>) => {
    const checked = event.target.checked;
    setSelectAll(checked);
    const updatedScalingComponentsData = scalingComponents.map(
      (updatedScalingComponentsDataItem) => ({
        ...updatedScalingComponentsDataItem,
        isChecked: checked,
      })
    );
    setScalingComponents(updatedScalingComponentsData);
  };

  const SIZE_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

  const [sizePerPage, setSizePerPage] = useState(SIZE_PER_PAGE_OPTIONS[0]);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPage, setTotalPage] = useState(1);

  useEffect(() => {
    setTotalPage(Math.ceil(scalingComponents.length / sizePerPage) || 1);
  }, [scalingComponents, currentPage, totalPage, sizePerPage]);

  const handleSizePerPage = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const newItemsPerPage = parseInt(event.target.value, 10);
    setSizePerPage(newItemsPerPage);
    setCurrentPage(1);
  };

  const handleCurrentPage = (newPage: number) => {
    setCurrentPage(newPage);
  };

  useEffect(() => {
    setCurrentPage(page || 1);
    setSizePerPage(size || SIZE_PER_PAGE_OPTIONS[0]);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, size]);

  useEffect(() => {
    if (currentPage > totalPage || currentPage < 1) {
      setCurrentPage(1);
    }
    router.push(
      `/app/scaling-components?page=${currentPage}&size=${sizePerPage}`
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scalingComponents, currentPage, totalPage, sizePerPage, searchParams]);

  const [detailsModalFlag, setDetailsModalFlag] = useState(false);

  const onClickDetails = (
    scalingComponentsItem: ScalingComponentDefinitionEx | undefined
  ) => {
    setScalingComponentsItem(scalingComponentsItem);
    setDetailsModalFlag(true);
  };

  const [fetchFlag, setFetchFlag] = useState(false);

  useEffect(() => {
    if (fetchFlag) {
      fetchScalingComponents();
      setFetchFlag(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fetchFlag]);

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
            type="OUTER"
            title="Scaling Components"
            right={
              <div className="flex items-center">
                <button
                  className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
                  onClick={() => onClickDetails(undefined)}
                >
                  ADD COMPONENT
                </button>
              </div>
            }
          />
          <div className="flex w-full flex-col">
            <TableComponent
              data={scalingComponents}
              setData={setScalingComponents}
              /*  */
              selectAll={selectAll}
              handleSelectAll={handleSelectAll}
              /*  */
              sizePerPageOptions={SIZE_PER_PAGE_OPTIONS}
              sizePerPage={sizePerPage}
              handleSizePerPage={handleSizePerPage}
              /*  */
              currentPage={currentPage}
              totalPage={totalPage}
              handleCurrentPage={handleCurrentPage}
              /*  */
              onClickDetails={onClickDetails}
            />
          </div>
        </div>
      </div>
      {detailsModalFlag ? (
        <ScalingComponentDetailDrawer
          scalingComponentsItem={scalingComponentsItem}
          setDetailsModalFlag={setDetailsModalFlag}
          setFetchFlag={setFetchFlag}
        />
      ) : null}
    </main>
  );
}
