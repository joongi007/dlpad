# S07: 개발 설치 화면 재시작 후 기존 bundle 검사

- 상태: done (기존 설치 재검증·명시 제거와 현재 Windows 개발 범위 인수 완료; 정식 installer/전체 P0 제외)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 worker service·installer UI·통합 검증/문서; metrics_recovery — /tmp 기존 bundle 검사 API·UI 계약 후보. 프로젝트 수정은 root만 한다.
- 선행 작업: [명시 제거 UI](S07-windows-native-development-removal-ui.md)의 인수, [제거 엔진](S07-windows-explicit-user-bundle-removal.md)의 현재 bundle/journal 검사.

## 사용자 결과

설치 화면을 다시 실행해 사용자가 지정한 기존 bundle과 외부 journal을 읽어 검증한 뒤 명시 제거 확인을 사용할 수 있다. 설치 source와 원 manifest가 없어도 현재 설치의 검증된 metadata/journal을 대조한다. startup에는 검증·제거를 자동 실행하지 않는다.

## 계약과 범위

실제 수정 파일은 `crates/engine/src/app_install/local_prepare.rs`, `app_install.rs`(root), `crates/app/src/installer_main.rs`, `installer_catalog.rs` 및 새 private native 재시작 fixture/collector와 개발 안내다. 현재 설치 네 경로의 create-only 흐름을 유지하고 기존 검사 요청은 bundle/journal 두 명시 경로만 사용한다. Inspect 보고는 current observation이며 삭제 권한이 아니다. 한 worker·bounded 최종 queue·active-only timer·원 worker ACK를 유지하고 제거 확인 시 별도 fresh authorization을 한다. 새 schema/dependency/lock는 제안하지 않는다. default/browse/정식 packaging·upgrade는 후속이다.

## 인수 조건

실제 private desktop에서 첫 설치 UI를 자연 종료한 뒤 두 번째 UI에서 기존 bundle/journal 입력→명시 검사→확인 취소/확인 제거를 검증한다. 첫 실행의 원 ID·SHA와 source/journal/sentinel 보존을 별도 readonly native 검사한다. source/manifest 입력을 비워도 검사/제거 경로가 설치를 자동 시작하지 않아야 한다. unknown/foreign/torn/mismatched journal·경로수정·취소/Close ACK·기존 설치회귀와 Windows전체검사를 인수한다. 확인되지 않은 physical/clean OS/upgrade/서명과 전체 P0 완료로 확대하지 않는다.

## 검증과 handoff

아래 초기 조사·통합 기록은 당시 상태이며 최신 인수 결과는 후속 절을 따른다.

2026-10-06 현재 초기 verify_available=false로 새 창의 기존 설치 검사는 CLI fallback인 개발 UI 한계를 기록했다. 이번 slice는 원 source 경로에 의존하지 않는 현재 bundle/journal 검사 후보를 조사한다. 아직 API 확정·프로젝트 구현·native 실행은 하지 않았다. 기존 제거 UI source592/844인수와 negative 설치회귀·최신 비용 검증을 우선한다.

Root공유계약선택: 새module 대신 local_prepare.rs에 LocalInspectionRequest(bundle_root,journal_root)와inspect_local_existing_bundle을 추가하고 기존4pathAPI는동일private inspect_roots로위임한다. 실제기존검사본문은source/manifest를읽지않으므로UI만입력에불필요한의존이있었다. /tmp backend기존제어순서/보고를보존한후보와UItyped Prepare4/Verify2/Remove2 요청·idle검증버튼·현재검증뒤제거확인 snapshot을준비했다. source원bytespins를root가확인하고현재CPU비용/최종문서검사가끝난뒤통합한다. 새공유schema/dependency없음. wrongSHA관측도구의203비활성화조건은이후readonly검증버튼계약에맞춰갱신하고204비활성화를검사해야한다. 아직후속코드미통합/native실행미완료다.

2026-10-06 root통합: 이전source592/CPU0bf43b의18비용창과원30k합성/3DB를root/독립인수한뒤원source/draftSHA를대조하고typed2root서비스와UI·별도두process재시작observer를반영했다. 기존4path inspector도동일privatehelper로위임하고검사순서/caller제어/보고를보존한다. Install4/Verify2/Remove2 요청을분리했다. 무인자새창도readonly Verify를선택할수있지만startup자동worker/I/O는없다. 실패뒤203readonly검증활성/204제거비활성계약에맞춰wrongSHA·복사취소관측도구를갱신했다(진행중비활성조건은유지). 새schema/dependency/lock없음. 추가service실패/원source없는검사2회귀와전체Windows빌드·별도process실제재시작은아직미실행이다.

### 최신 전체 검사

Windows MSVC fmt/clippy/test/release 실제 4명령 exit 0. 41 suites에서 846 passed, 0 failed, 13 ignored. 주 담당과 독립 검토가 현재 소스 592개 전체 map 및 실행 파일 pin을 확인했다. 근거: `../validation/evidence/windows-msvc-native-existing-inspection-workspace-20261006-continue/root-review.json`과 `independent-review.json`. 이 전체 검사 시점에 실제 재시작과 새 CPU 성능은 별도 항목이었다. 재시작 결과는 다음 절에 기록한다.

### 실제 별도 프로세스 재시작

`restart-back`·`restart-remove` 각각 실제 두 프로세스를 순차 실행하고 자연 종료·Job0를 확인했다. 원본/명세 입력이 빈 새 창에서 두 경로만 입력한 검증과 확인 취소·실제 제거를 인수했다. 주 담당 추가 native 14명령에서 원본·journal ID/SHA 보존, 남은 bundle의 canonical/domain/phase binding과 제거 후 Windows 폴더 부재를 확인했고 PNG 6장을 직접 검토했다. 독립 검토를 보관했다. 근거: `../validation/evidence/windows-msvc-native-existing-inspection-restart-root-native-20261006-continue/root-review-with-pixel.json`. 모든 내부 파일 접근 부재·삭제된 metadata canonical 바이트·physical IME를 증명한 것은 아니다. 기존 negative/취소 회귀 및 최신 CPU 비용은 아직 별도 진행한다.

### 기존 제거 회귀

새 binary에서 확인 취소·정상 제거·foreign 거절·경로 수정 거절·원 worker 취소·Close ACK 6개 실제 GUI 시험(12명령)을 재실행했다. root 추가 native 46명령과 직접 PNG 7장, 독립 검토에서 원본/journal ID·SHA 보존 및 actual 외부 폴더 부재를 인수했다. 근거: `../validation/evidence/windows-msvc-native-existing-inspection-removal-six-root-native-20261006-continue/root-review-with-pixel.json`. 삭제한 canonical metadata 원바이트와 정확한 취소 engine phase는 독립 증명하지 않았다. 기존 설치 거절·wrongSHA·복사취소/Close 시험 4건은 실행 완료·추가 root 검토 중이며 최신 CPU 비용은 아직 미검증이다.

### 기존 설치 실패 회귀

기존 설치 거절·wrongSHA·복사 중 취소·복사 중 Close ACK 4건을 새 binary에서 재실행했다(실제 8명령). root 추가 20 native 명령에서 실제 원 journal/native binding·원 12파일 보존·해시 오류 시 새 root 부재·합성 128MiB 원본 prefix/zero padding 및 partial 보존을 확인했다. 직접 PNG 5장을 확인했고 독립 검토를 보관했다. 근거는 각 `windows-msvc-native-existing-inspection-bootstrap-{repeat,wrongsha,copy-cancel,copy-close}-real-20261006-continue` evidence의 root 보고와 `../validation/evidence/windows-msvc-native-existing-inspection-removal-six-root-native-20261006-continue/independent-bootstrap-negative-review.json`이다. wrongSHA의 원 creator ID 및 typed GUI ErrorKind는 증명하지 않았다. 현재 모든 GUI 12case/24 collector 명령·root 추가 native 80명령·PNG 18장 인수 완료다. 최신 CPU3×60/18창 측정은 별도 진행 중이다.

### 최신 성능·최종 인수

현재 CPU SHA `80b65756b8b5b0b9d84a4e1f250c5b371db15bafca695b8e26694966b369a01a`, 8,710,656B의 실제 3×60초/18고정 구간이 모두 CPU<1%·Job합계 WS<80MiB를 통과했다. root 원 kernel/user 100ns delta/duration 계산 최대 CPU0.936280887%·WS71,630,848B, 직접 PNG3장 및 원 합성30k파일/readonly DB3의 exact 경로·1만행·integrity/원SHA 보존을 독립 검토와 대조했다. 근거: `../validation/evidence/windows-msvc-native-existing-inspection-default-cost-real-20261006-continue/root-combined-review.json`과 `independent-review.json`. fixture에 외부 sentinel은 없으며 별도 native ID 재조회·thread attribution·whole-host quiet·physical IME/reader·clean OS를 증명하지 않았다. 이번 기능 범위는 완료다. installer 증가 1,024B(798,720→799,744B), 새 dependency/schema/lock 없음. 다음은 [현재 세 binary 개발 ZIP](S07-windows-native-installer-companion-package.md)의 실제 인수다. 정식 설치/upgrade/서명/legal/전체 P0는 미완료다.
