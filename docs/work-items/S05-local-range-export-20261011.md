# S05: 로컬 영상의 선택 구간 저장

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: root engine media_preview/local_export.rs·app 통합·공유 catalog·검증, timeline_precision 원cap 독립 검토 /tmp, diagnostic_inputs local_export_worker.rs /tmp 후보. 같은 파일 동시 수정 없음.
- 선행 작업: 로컬 Prepared 원 pin과 selection, 관리 도구 실행 scope

## 사용자 결과

드롭하거나 목록에서 선택한 로컬 영상의 구간을 기존 저장 상위 폴더 안의 새 폴더에 저장한다. 원 영상과 기존 결과를 덮어쓰지 않는다. 준비 중인 후보를 완료 기능으로 표시하지 않는다.

## 계약과 범위

Rust native, argv 실행, 긴 I/O worker, bounded completion와 원 process cleanup ACK. 기존 파일 삭제와 경로 기반 임시 정리를 도입하지 않는다. 부분 산출물은 보존하고 상태를 사용자에게 알린다. 원 source/parent/stage HANDLE의 현재 identity와 공유 정책을 실제 Windows에서 확인한다. 새 dependency/schema/Cargo 변경 없음. 인코딩 시간·결과 크기 상한은 실험 후 문서화한다.

## 인수 조건

합성 영상에서 선택 구간의 실제 길이와 읽을 수 있는 프레임, 긴 소스 후반 탐색, 기존 파일·원본 보존, 취소·오류·닫기 ACK, 관리 도구 lease, stale 결과의 생성 사실 보존을 검증한다. private GUI에서 저장 위치·상태·disabled·취소·성공을 확인한다.

## 검증과 handoff

2026-10-11 engine 및 worker는 /tmp 후보와 독립 검토 단계다. 실제 compile/Windows FFmpeg/GUI 인수 전이며 구현 완료로 주장하지 않는다.

Root 실제검증: 새 Windows 원cap 회귀5passed/0failed와 UI/worker 상태 회귀7passed/0failed, 원 natural exit0/Job0/privateclosed/inputdesktop unchanged. engine/UI/worker 통합은 완료했으나 actualFFmpeg와 private GUI는 다음 검증이다. 개발 상한은 구간10분/결과256MiB/encode120초·전체180초이며 긴 소스 후반의 입력seek를 사용한다. 상한을 화면에 안내하고 10분 초과는 실행 전 거절한다. FFmpeg -fs는 엄격한 disk cap이 아니며 실제 결과 size 검사 초과도 보존한다. 임시 stage hardlink는 원 final과 같은 bytes이며 자동 삭제하지 않는다. 후보 수정파일: engine media_preview/local_export.rs, app local_export_worker.rs/local_export_ui.rs/accessible.rs/media_workspace.rs/catalog.rs/두 native host. 새 의존성·schema·lock 변경 없음.

Actual FFmpeg 첫 두 시도는 encoder libx264 부재로 실패했다. 공급한 LGPL FFmpeg에 외부 x264를 있다고 가정한 후보가 원인이며 frame/cap 성공으로 숨기지 않는다. root는 추가 GPL encoder 공급 대신 FFmpeg 내장 MPEG-4 video와 AAC, q:v3로 수정했다. 입력seek·출력크기/시간/원cap 계약은 유지한다. 실제 media 검증과 일반 플레이어 호환성은 구분하며 해당 codec 선택은 개발 범위다. Test-only 오류 계측은 원인 확인 뒤 제거했다. 두 실패 proof 보존.

Root 실제 내장코덱 회귀: `python3 /tmp/dlpad-local-real-test.py actual_synthetic_range_export_verifies_blue_clip_and_preserves_source_and_existing_outputs .../windows-msvc-local-export-real-native-codec-20261011` 1passed/0failed, 원natural0/Job0/privateclosed. 실제2초원본에서1100..1800ms 길이·파란frame·stage/final동일NativeID·기존외부root/final/stage원bytes·사전취소/기한경과의 생성없음과원본bytes를 확인했다. 8시간 소스와 inflight 취소·actual GUI는 이 시험으로 완료하지 않는다. [후보 ADR](../adr/0007-local-range-export.md).

후반 실제 8h fixture(57,600 frames, 778,515 B) 생성 8.82초. 첫 export는 낮은 FPS의 encoder timebase 반올림 때문에 InvalidMetadata로 실패하고 stage를 보존했다. 별도 실제 세 대안 비교에서 fine timebase 1:1000/passthrough는 원 video epoch .250초를 유지하며 3frame/1.500초였다. video만 0으로 reset하는 setpts는 A/V phase를 변경하므로 채택하지 않는다. 30fps 변환은 45frame으로 비용 증가/고FPS정보 손실을 초래하여 채택하지 않는다. 공통 timeline을 유지하는 fine timebase 후보를 통합했고 새 whole/실제 후반/A/V 검증 전이다. 실제 960×640 PNG에서 local 상태의 URL 입력과 중복 안내가 구간 입력을 밀어내는 것을 확인하여 source 변경 버튼으로 정리했다.

새 실제 fine-timebase 후반 시험 1passed/0failed, 처리 0.263302초, 원본 SHA 보존·파란 프레임·1.5초·stage/final 정체성 통과. 짧은 합성 영상도 정확한 필터 재실행에서 1passed/0failed(앞선 잘못된 이름 실행은 0tests이며 성공 근거가 아니다). 영상/음성 공통 시간 정렬과 의도된 200ms audio offset 합성 시험을 추가하고 실행 준비 중이다. local 상태 시간 입력을 오른쪽 저장 설정으로 옮겨 작은 창에서 접근성을 개선했다.

실제 4초 2fps flash/tone fixture 두 개와 750/1000ms 시작 범위 총 네 가지 A/V 측정 회귀 1passed/0failed. 영상 epoch drift0ms, 음성 onset drift 최대6ms, 의도된200ms offset 유지 및 원SHA 보존. 30ms 회귀 상한은 이 합성 AAC 케이스의 개발 인수이며 모든 codec/사이트에 대한 sample-accurate 계약은 아니다. 첫 host설정오류 실행은 build실패로 보존했고 원MSVC설정 재실행 증거만 사용한다.
