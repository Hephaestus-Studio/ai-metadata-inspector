import React, { useRef, useState } from 'react'
import { Badge, Box, Button, Loader, Stack, Tabs, Text } from '@mantine/core'
import { useDisclosure } from '@mantine/hooks'
import { notifications } from '@mantine/notifications'
import {
  IconCamera,
  IconCertificate,
  IconCheck,
  IconEraser,
  IconFileCode,
  IconSettings,
  IconSparkles,
} from '@tabler/icons-react'
import JSZip from 'jszip'

import { Header } from '@/components/Header'
import { ImageDropzone } from '@/components/ImageDropzone'
import type { BatchItem } from '@/components/BatchQueue'
import { RiskScoreCard } from '@/components/RiskScoreCard'
import { AiTab } from '@/components/AiTab'
import { ExifTab } from '@/components/ExifTab'
import { C2paTab } from '@/components/C2paTab'
import { RawChunksTab } from '@/components/RawChunksTab'
import { CleanModal } from '@/components/CleanModal'
import { cleanImage, inspectImage } from '@/services/wasmService'
import { type CleanOptions, DEFAULT_CLEAN_OPTIONS } from '@/types/metadata'
import classes from './App.module.css'

export const App: React.FC = () => {
  const [items, setItems] = useState<BatchItem[]>([])
  const [selectedIndex, setSelectedIndex] = useState<number>(0)
  const [inspecting, setInspecting] = useState<boolean>(false)
  const [cleaning, setCleaning] = useState<boolean>(false)
  const [cleaningBatch, setCleaningBatch] = useState<boolean>(false)
  const [activeTab, setActiveTab] = useState<string | null>('ai')

  const [cleanModalOpened, { open: openCleanModal, close: closeCleanModal }] =
    useDisclosure(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const selectedItem = items[selectedIndex]
  const currentReport = selectedItem?.report

  // Auto-detect optimal tab based on report data
  const getOptimalTab = (
    rep?: import('@/types/metadata').ImageMetadataReport,
  ): string => {
    if (!rep) return 'ai'
    if (rep.ai?.prompt || rep.ai?.platform) return 'ai'
    if (rep.exif && rep.exif.all_tags.length > 0) return 'exif'
    if (rep.c2pa?.has_c2pa) return 'c2pa'
    if (rep.raw_chunks_found.length > 0) return 'raw'
    return 'ai'
  }

  // Handle files selected from dropzone or file picker
  const handleFilesSelected = async (files: File[]) => {
    if (files.length === 0) return

    setInspecting(true)
    const newItems: BatchItem[] = []

    for (const file of files) {
      const previewUrl = URL.createObjectURL(file)
      const id = `${file.name}-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`

      try {
        const report = await inspectImage(file)
        newItems.push({ id, file, previewUrl, report })
      } catch (err) {
        console.error('Failed to inspect file:', err)
        notifications.show({
          title: 'Inspection Warning',
          message: `Could not parse metadata for ${file.name}`,
          color: 'orange',
        })
        newItems.push({ id, file, previewUrl })
      }
    }

    setItems((prev) => {
      const updated = [...prev, ...newItems]
      return updated
    })

    if (items.length === 0 && newItems.length > 0) {
      setSelectedIndex(0)
      if (newItems[0].report) {
        setActiveTab(getOptimalTab(newItems[0].report))
      }
    }

    setInspecting(false)

    notifications.show({
      title: 'Inspection Complete',
      message: `Processed ${files.length} image(s) with WebAssembly!`,
      color: 'teal',
      icon: <IconCheck size={16} />,
    })
  }

  // Quick Clean & Download single file
  const handleQuickClean = async (
    options: CleanOptions = DEFAULT_CLEAN_OPTIONS,
  ) => {
    if (!selectedItem) return
    setCleaning(true)

    try {
      const result = await cleanImage(selectedItem.file, options)
      if (result.success) {
        const blob = new Blob([result.cleaned_bytes as unknown as BlobPart], {
          type: selectedItem.file.type || 'image/png',
        })
        const downloadUrl = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = downloadUrl
        const baseName = selectedItem.file.name.replace(/\.[^/.]+$/, '')
        const ext = selectedItem.file.name.split('.').pop() || 'png'
        a.download = `${baseName}_cleaned.${ext}`
        a.click()
        URL.revokeObjectURL(downloadUrl)

        notifications.show({
          title: 'Metadata Stripped Losslessly',
          message: `Saved ${result.percentage_reduced.toFixed(1)}% (${(
            result.bytes_removed / 1024
          ).toFixed(1)} KB removed)`,
          color: 'teal',
          icon: <IconCheck size={16} />,
        })
      } else {
        throw new Error(result.error || 'Sanitization failed')
      }
    } catch (err) {
      notifications.show({
        title: 'Cleaning Error',
        message: String(err),
        color: 'red',
      })
    } finally {
      setCleaning(false)
      closeCleanModal()
    }
  }

  // Batch Clean all files and download as ZIP
  const handleBatchCleanAndDownloadZip = async () => {
    if (items.length === 0) return
    setCleaningBatch(true)

    try {
      const zip = new JSZip()

      for (let i = 0; i < items.length; i++) {
        const item = items[i]
        const result = await cleanImage(item.file, DEFAULT_CLEAN_OPTIONS)
        if (result.success) {
          const baseName = item.file.name.replace(/\.[^/.]+$/, '')
          const ext = item.file.name.split('.').pop() || 'png'
          zip.file(`${baseName}_cleaned.${ext}`, result.cleaned_bytes)
        }
      }

      const zipBlob = await zip.generateAsync({ type: 'blob' })
      const url = URL.createObjectURL(zipBlob)
      const a = document.createElement('a')
      a.href = url
      a.download = `cleaned_metadata_images_${Date.now()}.zip`
      a.click()
      URL.revokeObjectURL(url)

      notifications.show({
        title: 'Batch Clean Successful',
        message: `Exported ${items.length} sanitized images to ZIP archive!`,
        color: 'teal',
        icon: <IconCheck size={16} />,
      })
    } catch (err) {
      notifications.show({
        title: 'Batch Error',
        message: `Failed to create ZIP: ${err}`,
        color: 'red',
      })
    } finally {
      setCleaningBatch(false)
    }
  }

  const handleRemoveItem = (index: number) => {
    URL.revokeObjectURL(items[index].previewUrl)
    const updated = items.filter((_, idx) => idx !== index)
    setItems(updated)
    if (selectedIndex >= updated.length) {
      setSelectedIndex(Math.max(0, updated.length - 1))
    }
  }

  const handleClearAll = () => {
    items.forEach((it) => URL.revokeObjectURL(it.previewUrl))
    setItems([])
    setSelectedIndex(0)
  }

  return (
    <Box className={classes.appRoot}>
      {/* Dynamic Animated Atmospheric Mesh Blobs */}
      <div className={classes.liquidAuroraMesh} aria-hidden="true">
        <div className={`${classes.auroraBlob} ${classes.blobCyan}`} />
        <div className={`${classes.auroraBlob} ${classes.blobPurple}`} />
        <div className={`${classes.auroraBlob} ${classes.blobPink}`} />
        <div className={`${classes.auroraBlob} ${classes.blobEmerald}`} />
      </div>

      <div className={classes.contentWrapper}>
        <Header
          items={items}
          selectedIndex={selectedIndex}
          onSelectIndex={(idx) => {
            setSelectedIndex(idx)
            if (items[idx]?.report) {
              setActiveTab(getOptimalTab(items[idx].report))
            }
          }}
          onRemoveItem={handleRemoveItem}
          onClearAll={handleClearAll}
          onAddMore={() => fileInputRef.current?.click()}
          onBatchCleanAndDownloadZip={handleBatchCleanAndDownloadZip}
          cleaningBatch={cleaningBatch}
        />

        <main className={classes.mainContainer}>
          {/* Hero Section */}
          {items.length === 0 && (
            <div className={classes.heroSection}>
              <h1 className={classes.heroTitle}>
                Inspect AI Prompts &{' '}
                <span className={classes.gradientText}>Strip Privacy Tags</span>
              </h1>
              <p className={classes.heroDesc}>
                Deep extraction for Stable Diffusion, ComfyUI node graphs,
                NovelAI, EXIF GPS locations, and C2PA Content Credentials. 100%
                offline & lossless.
              </p>
            </div>
          )}

          {/* Upload Zone */}
          {items.length === 0 ? (
            <div className={classes.emptyStateWrapper}>
              <ImageDropzone
                onFilesSelected={handleFilesSelected}
                loading={inspecting}
              />

              {/* Feature Showcase Cards */}
              <div className={classes.featureGrid}>
                <div className={classes.featureCard}>
                  <div
                    className={`${classes.featureIcon} ${classes.featureIconAi}`}
                  >
                    <IconSparkles size={24} />
                  </div>
                  <h3 className={classes.featureTitle}>
                    AI Generation & Graphs
                  </h3>
                  <p className={classes.featureDesc}>
                    Extract Stable Diffusion prompts, ComfyUI node workflows,
                    NovelAI, FLUX & SDXL generation settings.
                  </p>
                </div>

                <div className={classes.featureCard}>
                  <div
                    className={`${classes.featureIcon} ${classes.featureIconExif}`}
                  >
                    <IconCamera size={24} />
                  </div>
                  <h3 className={classes.featureTitle}>
                    EXIF & GPS Geolocation
                  </h3>
                  <p className={classes.featureDesc}>
                    Uncover camera hardware, lens profiles, exposure details,
                    and embedded GPS coordinate radar.
                  </p>
                </div>

                <div className={classes.featureCard}>
                  <div
                    className={`${classes.featureIcon} ${classes.featureIconClean}`}
                  >
                    <IconEraser size={24} />
                  </div>
                  <h3 className={classes.featureTitle}>
                    Lossless Privacy Scrubber
                  </h3>
                  <p className={classes.featureDesc}>
                    Strip tracking tags and metadata chunks in milliseconds with
                    0% image re-compression.
                  </p>
                </div>
              </div>
            </div>
          ) : (
            <div>
              {/* Hidden File Input for Add More */}
              <input
                type="file"
                ref={fileInputRef}
                style={{ display: 'none' }}
                multiple
                accept="image/png,image/jpeg,image/webp"
                onChange={(e) => {
                  if (e.target.files) {
                    handleFilesSelected(Array.from(e.target.files))
                    e.target.value = ''
                  }
                }}
              />

              {/* Main Dual-Panel Dashboard */}
              {selectedItem && (
                <div className={classes.dashboardGrid}>
                  {/* Left Column: Unified Ergonomic Media & Action Workbench */}
                  <div className={classes.leftPanel}>
                    <div className={classes.previewCard}>
                      <div className={classes.imagePreviewWrapper}>
                        <img
                          src={selectedItem.previewUrl}
                          alt={selectedItem.file.name}
                          className={classes.imagePreview}
                        />
                      </div>

                      <div className={classes.fileInfoRow}>
                        <div style={{ minWidth: 0, flex: 1 }}>
                          <Text
                            size="sm"
                            fw={700}
                            truncate
                            title={selectedItem.file.name}
                          >
                            {selectedItem.file.name}
                          </Text>
                          {currentReport && (
                            <Text size="xs" c="dimmed" mt={2}>
                              {currentReport.width} × {currentReport.height} px
                              •{' '}
                              {(currentReport.file_size_bytes / 1024).toFixed(
                                1,
                              )}{' '}
                              KB
                            </Text>
                          )}
                        </div>
                        <Badge
                          variant="filled"
                          color="indigo"
                          size="sm"
                          style={{ flexShrink: 0 }}
                        >
                          {currentReport?.format || 'IMAGE'}
                        </Badge>
                      </div>

                      {/* Embedded Privacy Risk Assessment */}
                      {currentReport?.risk_report && (
                        <div className={classes.riskScoreWrapper}>
                          <RiskScoreCard
                            riskReport={currentReport.risk_report}
                            embedded
                          />
                        </div>
                      )}

                      {/* Ergonomic Action Buttons */}
                      <Stack gap="xs" mt="xs">
                        <Button
                          className={classes.quickCleanBtn}
                          size="md"
                          loading={cleaning}
                          leftSection={<IconEraser size={18} />}
                          onClick={() => handleQuickClean()}
                          fullWidth
                        >
                          Lossless Clean & Save
                        </Button>

                        <Button
                          color="gray"
                          variant="light"
                          size="xs"
                          leftSection={<IconSettings size={15} />}
                          onClick={openCleanModal}
                          fullWidth
                        >
                          Custom Stripping Options
                        </Button>
                      </Stack>
                    </div>
                  </div>

                  {/* Right Column: Metadata Detail Tabs */}
                  <div className={classes.rightPanel}>
                    <div className={classes.tabsCard}>
                      {currentReport ? (
                        <Tabs value={activeTab} onChange={setActiveTab}>
                          <Tabs.List mb="md">
                            <Tabs.Tab
                              value="ai"
                              leftSection={<IconSparkles size={16} />}
                              rightSection={
                                currentReport.ai?.prompt ||
                                currentReport.ai?.platform ? (
                                  <Badge size="xs" color="cyan">
                                    Found
                                  </Badge>
                                ) : null
                              }
                            >
                              AI Generation
                            </Tabs.Tab>

                            <Tabs.Tab
                              value="exif"
                              leftSection={<IconCamera size={16} />}
                              rightSection={
                                currentReport.exif ? (
                                  <Badge size="xs" color="indigo">
                                    {currentReport.exif.all_tags.length}
                                  </Badge>
                                ) : null
                              }
                            >
                              EXIF & GPS
                            </Tabs.Tab>

                            <Tabs.Tab
                              value="c2pa"
                              leftSection={<IconCertificate size={16} />}
                              rightSection={
                                currentReport.c2pa?.has_c2pa ? (
                                  <Badge size="xs" color="teal">
                                    Verified
                                  </Badge>
                                ) : null
                              }
                            >
                              C2PA Credentials
                            </Tabs.Tab>

                            <Tabs.Tab
                              value="raw"
                              leftSection={<IconFileCode size={16} />}
                              rightSection={
                                currentReport.raw_chunks_found.length > 0 ? (
                                  <Badge size="xs" color="gray">
                                    {currentReport.raw_chunks_found.length}
                                  </Badge>
                                ) : null
                              }
                            >
                              Raw Chunks
                            </Tabs.Tab>
                          </Tabs.List>

                          <Tabs.Panel value="ai">
                            <AiTab
                              ai={currentReport.ai}
                              report={currentReport}
                              onNavigateTab={setActiveTab}
                            />
                          </Tabs.Panel>

                          <Tabs.Panel value="exif">
                            <ExifTab
                              exif={currentReport.exif}
                              report={currentReport}
                              onNavigateTab={setActiveTab}
                            />
                          </Tabs.Panel>

                          <Tabs.Panel value="c2pa">
                            <C2paTab
                              c2pa={currentReport.c2pa}
                              report={currentReport}
                              onNavigateTab={setActiveTab}
                            />
                          </Tabs.Panel>

                          <Tabs.Panel value="raw">
                            <RawChunksTab
                              chunks={currentReport.ai?.raw_text_chunks || []}
                              rawChunksFound={currentReport.raw_chunks_found}
                              report={currentReport}
                              onNavigateTab={setActiveTab}
                            />
                          </Tabs.Panel>
                        </Tabs>
                      ) : (
                        <Stack align="center" justify="center" h={300}>
                          <Loader size="lg" color="indigo" />
                          <Text size="sm" c="dimmed">
                            Analyzing image metadata via WebAssembly...
                          </Text>
                        </Stack>
                      )}
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}
        </main>

        {/* Granular Clean Settings Modal */}
        <CleanModal
          opened={cleanModalOpened}
          onClose={closeCleanModal}
          onCleanAndDownload={handleQuickClean}
          loading={cleaning}
        />
      </div>
    </Box>
  )
}

export default App
