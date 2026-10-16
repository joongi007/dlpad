# S00: 페이지 이동 문구 정리

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — crates/app/src/catalog.rs와 다음 현재 native 시험 source, 이 문서
- 선행 작업: 현재 시각 감사

## 사용자 결과

파일 목록과 작업 이력에서 내부 batch 상한 대신 다음 페이지로 이동한다는 의미를 읽는다.

## 계약과 범위

PRODUCT의 사용자 행동 우선 문구 계약을 따른다. 중앙 NEXT/HISTORY_NEXT만 다음 페이지로 바꾸며 조회128개 상한·가상화·상태·페이지 정책은 그대로 유지한다. 현재 native 시험의 버튼 조회 문구도 맞추며 이전 캡처와 archive inputs는 변경하지 않는다.

## 인수 조건

현재 UI에서 다음 페이지 버튼을 찾고 기존 다음128번째 파일 이동을 수행할 수 있다. 빈 목록에서는 기존 비활성 상태를 유지한다. 단순 문구를 복제하는 단위 테스트를 추가하지 않고 실제 목록 회귀와 캡처로 확인한다.

## 검증과 handoff

시각 감사에서 batch128 구현 상세가 기본 UI에 노출되는 것을 확인했다. 수정 파일 목록:

- scripts/windows_library_loaded_probe.cs
- scripts/windows_library_periodic_functional_probe.cs
- scripts/windows_library_periodic_thread_cpu_probe.cs

- 중앙 NEXT/HISTORY_NEXT를 다음 페이지로 수정하고 현재 native source3개의 Button 조회도 맞췄다. 조회128개 상한과 기존 media-00128.mp4 도달 assertion은 유지했다. 과거 native archive inputs와 캡처는 보존한다. 최신 전체와 실제 목록 화면은 진단 재배치 통합 이후 묶어서 검증한다.

- 진단 UI 최신1173 통합 이후 diagnostic_inputs에 /tmp 시험 runner 준비를 위임했다. 담당 파일은 /tmp 후보만이며 프로젝트/Cargo/native 실행은 root가 독점한다. 기존128개 상한과 media-00128.mp4 도달 검증을 유지한다. 실제 실행·화면 인수는 아직 남는다.

- 最新loaded 실행14534 exit2 실패를 보존했다: windows-msvc-library-next-page-current-20261012. 원 observer의 Invoke pattern unavailable/disabled에서 중단되어 다음 페이지 성공·빈 상태를 인수하지 않는다. 실패 단계/name/role 정보가 없어 제품 결함 여부는 미확정이며 /tmp instrumentation 후보를 diagnostic_inputs에 위임했다. 원 source/128/10k/DB/Job 기준은 유지한다. 사용자 데스크톱은 변경하지 않았으며 observer 내부 owned Job 강제 정리가 보고돼 정상 종료 인수로 계산하지 않는다.

- instrumentation v2 actual80468 exit2로 실패 조작을 확정했다: navigate / 파일 목록 / ControlType.Button / enabled=True / invoke=False. 페이지 이동에 도달하기 전 탐색 pattern 불일치다. 원 raw native.log는 CP949 바이트를 포함하며 그대로 보존했다. 현재 선택 탐색의 실제 Toggle/SelectionItem 의미를 원 UI와 Windows adapter source에서 확인해 적절한 semantic 시험 경로를 검토한다. 기준을 완화하거나 global/pointer fallback을 추가하지 않는다.

- 실제 semantic80782 exit0 및 독립10k reader77546 exit0: Toggle Off→On·page1 media00000→page2 media00128/oldrow 부재·10000 namespace/bytes/SHA·sentinel·readonly DB integrity/수동 변경/전체count를 대조했다. root가page1/page2PNG직접 검토해 caption 다음 페이지와 실제 행 이동을 확인했고 root-independent-review.json/root-scoped-pixel-review.json을 기록했다. 빈 상태 최신 검증은 별도이며 완료 상태는 아직 바꾸지 않는다. 원 실패2개는 보존한다.

- 새 picker source 전체1193 통과 이후 별도빈목록native87095를 실행 중이다. 기존 loaded10k 증거는 이전1173 snapshot으로 보존하며 최신source의 loaded 시험으로 소급하지 않는다. 빈상태에서는 readonly DB0·media namespaceempty·sentinel·Nextdisabled/행부재/navToggleOn·배율별스크롤을 별도 수집한다.

- 빈목록87095는 native 시작 전 expected-tools.json /tmp fixture 의존성 누락으로 exit1했다. 원 출력은 아직 생성하지 않았고 root source 변화 없음. release_preparation에 정확한 원 의존성 SHA/전체 목록 확인을 요청했다. 이 preflight 실패를 빈목록 UI 실패나 성공으로 계산하지 않는다.

- 검증된 semantic observer를 현재 scripts/windows_library_loaded_probe.cs에 통합하고 대응 windows_msvc_library_loaded_native_probe.py의 C# UTF8 codepage를 명시했다. 원 archive inputs/실패2개/실제 semantic1173 증거는 그대로다. 앞으로 repository 기본 loaded 시험도 선택 탐색의 Toggle Off→On과 나머지 action Invoke 계약을 사용하며 128/10k/DB/Job 기준을 유지한다. 수정 파일 범위를 해당 collector로 추가한다. 이 스크립트 통합은 Rust 전체 source map에 포함되지 않으므로 별도 observer/collector SHA를 이후 native 실행에서 확인한다.

- missing expected-tools.json을 원empty-scoped-v3의1384바이트/SHA b531dcc7cfbcd8f0a54d9ec079feb2796574fb6c02c15f60f59fa9cc90b359ba로 create-only 복사하고 전체sibling 의존성을 확인했다. 원collect/observer 변경 없이 실제빈목록63711 실행 중이다. 독립reviewer는 /tmp/dlpad-library-empty-pagination-independent-20261012에 준비했다. 기본 loaded collector 문법은 /tmp cfile 출력으로 py_compile 통과했다. 상대경로 __pycache__ 쓰기는 read-only 오류였고 기존 캐시를 수정하거나 삭제하지 않았다.

- 최신1193 source 빈목록63711 exit0, 독립60257 exit0. 실제18matrix/35PNG·Next disabled/empty Text/row0/navToggleOn·readonlyDB0/empty media/sentinel·theme 재시작/Job 자연 종료를 대조했다. root가960100와800200 최초/선택Dark/restartDark after PNG를 직접 확인해 빈안내 전체와 비활성 다음 페이지를 scoped 인수했다. 별도 root pixel/independent 증거를 기록했다. 저장소에 통합한 loaded collector를 같은1193 source로 재실행해 script 통합까지 최종검증한다.

- 저장소 기본 loaded collector actual50990 exit0: 최신1193 source에서 실제page1/media00000→page2/media00128를 확인했다. root가두PNG직접 확인했고 독립 raw/전체10k file bytes/DB의 효율적 bounded reader 후보를 준비 중이다. 다른 source 검증을 섞지 않고 이 최신 repository-run의 독립 검토 이후 scoped 완료를 결정한다.

- 최신 repository loaded50990 exit0/독립35512 exit0: current1193 원source/CPU·원observercopy/collector SHA·Toggle Off→On·page1/2·전체10000files FD/bytes/SHA/namespace·readonlyDB integrity/currentchanges·sentinel/Job 자연 종료를 대조했다. root PNG2개 직접검수와 결과를새evidence에기록했다. 최신빈상태18matrix/35PNG 독립과도결합해 이 문구·다음페이지·빈상태 수직 작업을 scoped 완료한다. 전체 UI/UX·watchOnly·물리 입력·새 설치본 업데이트는 이 완료에 포함하지 않는다.

- 보조 Linux GUI probe의 기존 caption 참조도 cpu_library_probe.py/cpu_history_probe.py에서 다음 페이지로 일치시켰다. 수정 범위에 두 파일을 추가하며 /tmp cfile Python syntax만 통과했다. Linux GUI 실행/OS 인수는 현재 Windows 우선 범위에서 수행하지 않았고 완료로 주장하지 않는다. archived inputs는 변경하지 않았다.
