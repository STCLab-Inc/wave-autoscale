'use client';

import React, { useEffect, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';

import ScalingComponentService from '@/services/scaling-component';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

import ContentHeader from '../content-header';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';
import ScalingComponentDetailDrawer from './scaling-component-drawer';

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
  }, []);

  const [checkAllFlag, setCheckAllFlag] = useState(false);

  const handleCheckAllChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const checked = event.target.checked;
    setCheckAllFlag(checked);
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

  const startIndex = (currentPage - 1) * sizePerPage;
  const endIndex = startIndex + sizePerPage;
  const visibleScalingComponents = scalingComponents.slice(
    startIndex,
    endIndex
  );

  const handleSizePerPageChange = (
    event: React.ChangeEvent<HTMLSelectElement>
  ) => {
    const newItemsPerPage = parseInt(event.target.value, 10);
    setSizePerPage(newItemsPerPage);
    setCurrentPage(1);
  };

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
  };

  useEffect(() => {
    setCurrentPage(page || 1);
    setSizePerPage(size || SIZE_PER_PAGE_OPTIONS[0]);
  }, [page, size]);

  useEffect(() => {
    if (currentPage > totalPage) {
      setCurrentPage(1);
    }
    router.push(
      `/app/scaling-components?page=${currentPage}&size=${sizePerPage}`
    );
  }, [scalingComponents, currentPage, totalPage, sizePerPage]);

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
  }, [fetchFlag]);

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
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
            <div className="flex items-center justify-end px-8 py-4">
              <div className="mr-2 flex items-center">
                <label className="select-group-sm">
                  <select
                    value={sizePerPage}
                    onChange={handleSizePerPageChange}
                    className="focus:outline-noneselect select-sm max-w-[130px] cursor-pointer rounded-md border border-gray-200 px-2"
                  >
                    {SIZE_PER_PAGE_OPTIONS.map((option) => (
                      <option key={option} value={option}>
                        {option}
                      </option>
                    ))}
                  </select>
                </label>
              </div>

              <div className="mx-2 flex items-center justify-center">
                <span className="px-2 text-center text-sm">
                  {currentPage} / {totalPage}
                </span>
              </div>

              <div className="ml-2 flex h-8 items-center">
                <button
                  className={
                    currentPage === 1
                      ? 'mr-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                      : 'mr-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50'
                  }
                  onClick={() => handlePageChange(currentPage - 1)}
                  disabled={currentPage === 1}
                >
                  PREVIOUS
                </button>
                <button
                  className={
                    currentPage && totalPage && currentPage !== totalPage
                      ? 'ml-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50'
                      : 'ml-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                  }
                  onClick={() => handlePageChange(currentPage + 1)}
                  disabled={currentPage === totalPage}
                >
                  NEXT
                </button>
              </div>
            </div>

            <table className="flex w-full flex-col">
              <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-75 py-0 font-bold text-gray-800">
                <tr className="flex h-full w-full px-8">
                  <th className="mr-4 flex h-full flex-1 items-center">
                    <label className="flex h-full items-center">
                      <input
                        type="checkbox"
                        className="checkbox"
                        checked={checkAllFlag}
                        onChange={handleCheckAllChange}
                      />
                    </label>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-8 items-center">
                    <span className="flex items-center break-words">
                      Scaling Component ID
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-8 items-center">
                    <span className="flex items-center break-words">
                      Scaling Component Kind
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-8 items-center">
                    <span className="flex items-center break-words">
                      Metadata Values
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-1 items-center">
                    <span className="flex items-center break-words">
                      Actions
                    </span>
                  </th>
                </tr>
              </thead>
              <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between py-0 text-gray-800">
                {visibleScalingComponents.map(
                  (componentsItem: ScalingComponentDefinitionEx) => (
                    <tr
                      key={componentsItem.db_id}
                      className="flex w-full border-b px-8 py-4"
                    >
                      <td className="mr-4 flex h-full flex-1 items-start">
                        <label className="flex items-center">
                          <input
                            type="checkbox"
                            className="checkbox"
                            checked={componentsItem.isChecked}
                            onChange={(event) => {
                              const checked = event.target.checked;
                              const updatedComponents = scalingComponents.map(
                                (item) =>
                                  item.id === componentsItem.id
                                    ? { ...item, isChecked: checked }
                                    : item
                              );
                              setScalingComponents(updatedComponents);
                            }}
                          />
                        </label>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-8 items-start">
                        <div className="flex items-center break-all">
                          {componentsItem.id}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-8 items-start">
                        <div className="flex items-center break-all">
                          {componentsItem.component_kind}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-8 items-start">
                        <div className="flex flex-col items-center">
                          {renderKeyValuePairsWithJson(
                            JSON.stringify(componentsItem.metadata),
                            true
                          )}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-1 items-start">
                        <div className="flex items-center">
                          <button
                            className="badge-info badge bg-[#99E3D0] px-2 py-3 text-white"
                            onClick={() => onClickDetails(componentsItem)}
                          >
                            Details
                          </button>
                        </div>
                      </td>
                    </tr>
                  )
                )}
              </tbody>
            </table>
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
