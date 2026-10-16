# S04: managed archive 설치·선택·제거 네이티브 UI 연결

- 상태: proposed
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R03, R11
- 담당 예정/수정 범위: root 통합 조정; accessible_tools.rs의 도구 worker/action/inventory, managed.rs의 검증된 공급 candidate, catalog.rs 중앙 문구. root engine package 설치·목록·선택·제거 API는 별도 담당. 이번 수정은 이 문서만
- 선행 작업: engine archive 설치·fullset selection/removal 검증, 정확 HTTPS 공급·trust/license/notice·target별 component evidence 채택

## 사용자 결과

도구 화면에서 앱이 제시한 버전·플랫폼·크기·공급/라이선스 정보를 보고 관리 package를 다운로드·검증·설치한다. 설치된 버전을 선택하고 비활성 관리 세트만 제거한다. 외부 실행 파일 선택을 유지한다. 임의 JSON manifest를 고르는 파일 GUI로 목표를 축소하지 않는다.

## 문맥과 기존 인터페이스

README→PRODUCT→CONTEXT→REQUIREMENTS R05/R08→ADR0003→DESIGN 및 INTEGRATIONS의 공급 gate를 읽었다. ADR의 관리 공급은 Proposed이며 사용자 확정이나 archive parser PASS를 공급 채택 완료로 표현하지 않는다. FFmpeg/ffprobe는 같은 배포 세트, Deno 및 bundled solver 선택도 설치 합계와 실행 argv 연결이 필요하다.

accessible_tools.rs는 sync_channel1 command/results와 기존 단일 worker를 보유한다. Action Refresh/Install/Select(InstallManifest)/Remove(InstallManifest)/Use, Outcome Inventory/Selected(PathBuf), cancel AtomicBool 및 Ack stopped/failed를 그대로 활용한다. perform은 ToolManager와 staging recovery를 worker에서 실행한다. 현재 candidate()는 yt-dlp 직접 executable, inventory와 Use는 yt-dlp만 하드코딩했다. show에서 공급 version/size/target을 표시하고 설치/선택/제거 버튼을 제공한다. UI thread에 package hash/HTTPS/archive I/O를 추가하지 않는다.

## 구현 공유 계약

검증된 공급 Candidate는 Direct(manifest,bytes)와 Package(ArchivePackage)의 typed 분기를 제공하고 고정 tool ID별 candidate만 선택한다. 공급 schema/Cargo/catalog는 root가 조정한다. Action은 candidate kind를 worker에 전달하거나 worker가 같은 고정 공급 목록에서 재검증한다. 임의 URL/archive/component 이름 사용자 입력은 없다. Package install은 기존 bounded HTTPS 다운로드→archive hash→flat component extraction→component version/notice→owned publish API를 호출한다. Install 성공 뒤 fullset active evidence를 확인한 후 refresh한다. 취소/실패는 기존 active·외부 설치를 보존한다.

목록은 metadata-only recorded_package 경로로 package 표시 정보를 얻고, 사용/선택은 actual fullset hash를 요구한다. package 여부를 manifest archive hash로 추정하지 않는다. 설치된 main InstallManifest에 대응한 package_installed 조회는 fullhash 비용이 있으므로 UI 목록용으로 호출하지 않고 metadata package summary를 engine inventory에서 제공해야 한다. Remove는 root 기존 remove package routing을 활용하고 활성 버전 거절/foreignfile 보존을 고정 catalog 오류로 보여준다. FFmpeg/ffprobe는 같은 package record로 묶고 main/aux/notice 역할을 표시한다.

Use는 yt-dlp 경로만 넘기는 현재 Outcome Selected를 tool별 선택 결과로 확장하는 공유 UI 계약이 필요하다. FFmpeg 선택은 다운로드 기존 ffmpeg 필드에 연결하고 Deno 선택은 검증된 runtime argv의 명시 경로에 연결해야 한다. 미연결 Deno 버튼을 설치 완료 사용자 기능으로 표시하지 않는다. 별도 diagnostics/tools worker의 상호 배제와 CloseWait의 cancel/worker acknowledgment는 보존한다.

## 공급·trust 선행 조건

HTTPS 공급 URL/tag/정확 archive SHA 및 bytes/component hash·bytes/license/notice와 OS target 고정 evidence가 필수다. yt-dlp signed checksum 검증과 Deno/FFmpeg archive 후보 상태는 구분한다. FFmpeg BtbN build는 codec/license/source/notice 및 공급 신뢰 결정 전 자동 설치를 활성화하지 않는다. 미지원 target/trust gate에서는 고정 설명과 disabled install로 표시하고 외부 경로 선택은 제공한다. 새 deps/최종 스택 채택은 이 작업이 정하지 않는다.

## 인수 조건

사설 GUI에서 tool별 공급 정보/disabled gate/설치 대기·취소·완료·실패/metadata 목록/선택/활성제거 거절/비활성 exactset 제거를 UIA와 PNG로 확인한다. HTTPS fixture는127.0.0.1 또는 ::1 private 시험만 사용하고 제품 웹 UI 서버를 만들지 않는다. 잘못된 archive/hash/foreignentry/version/부분압축/metadata/write/delete 권한 실패에서 이전 active·원 source·외부 sentinel 보존을 engine와 실제 UI에서 검사한다. 외부 경로 도구는 제거 버튼을 제공하지 않는다. 960×640 및800×600에서 핵심 동작이 scroll/focus로 도달하고 Tab/Enter/Space로 수행 가능해야 한다. busy 동안 mutation 중복을 거절하며 CloseWait는 owned worker 종료 확인 뒤 닫힌다. 실제 native 도구 component version/ffprobe compatibility를 합성 fixture와 공식 공급 측정으로 구분한다.

## 검증과 handoff

읽기·설계 문서만 작성했다. source freeze를 지켜 Rust/공유문서/Cargo/GUI/프로세스를 변경·실행하지 않았다. Linux/Windows engine 현재 검증과 공급 gate를 root가 확정한 뒤 파일 담당·공유 Candidate/selection outcome을 합의하여 구현한다. 이 문서는 제품 연결 완료 또는 P0 완료가 아니다.
