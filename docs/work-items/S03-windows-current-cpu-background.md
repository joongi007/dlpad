# S03: 최신 Windows CPU 앱의 백그라운드 숨김·복귀 검증

- 상태: completed
- 분류: verification
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — scripts/windows_cpu_background_native_probe.cs, scripts/windows_cpu_background_worker.cs, native 실행·통합 검증·문서. metrics_recovery — /tmp 초안 및 독립 검토만.
- 선행 작업: 최신 기본 CPU 빌드와 스캔·성능 인수. 현재 진행 중인 실제 스캔 시험 종료 후 native를 직렬 실행한다.

## 사용자 결과

합성 다운로드 중 창을 숨겨도 같은 worker가 진행하며, 같은 실행파일을 다시 실행하면 원 앱 창과 원 다운로드로 복귀하는지 확인한다.

## 계약과 범위

Rust 네이티브 앱의 제품 기능을 시험하며 C#는 외부 관측 도구와 합성 worker에만 사용한다. 새 SID 전용 임시 폴더·private desktop·원 app/worker/secondary HANDLE·생성 시각·실행파일 image·Job을 고정한다. 기본 앱에 인자 없이 실행하고 합성 URL example.invalid 및 고정 version/metadata/progress 출력만 사용한다. worker 원 실행파일을 전체 수명 SHA pin하며 readiness 파일은 32/64byte 상한을 적용한다. 숨김 전후 heartbeat와 원 창 복귀, Stop 후 종료를 관측한다. 사용자 Documents·실제 영상·계정·네트워크·전역 키보드·clipboard·visible desktop fallback은 사용하지 않는다.

## 인수 조건

원 앱 창 숨김·원 worker heartbeat 증가·secondary 자연 exit 0·원 창 복귀·같은 worker 생성 시각·image 유지·Stop 후 자연 종료와 Job0, private desktop 폐쇄 및 사용자 입력 desktop 불변을 확인한다. root는 PNG, 읽기 전용 DB의 같은 Job/Attempt 및 sentinel 보존을 별도로 대조한다. 관측만으로 실제 픽셀 또는 전체 P0 완료로 승격하지 않는다.

## 검증과 handoff

metrics_recovery 초안 보강 후 root가 프로젝트 시험 스크립트로 통합했다. 현재 컴파일·native 실행은 미검증이다. UIA 기한 초과 시 cooperative Stop, 원 Job 종료, bounded join 후 HANDLE 정리를 적용한다. thread가 종료하지 않으면 FAIL과 보존 상태를 기록하며 원 HANDLE을 살아 있는 thread 아래에서 닫지 않는다. 외부 host 기한은 여전히 필요하다. 기본 Windows 공급·물리 IME·실제 도구 다운로드 및 숨긴 채 완료는 별도 인수다.

root가 `/tmp` runner를 `scripts/windows_cpu_background_native_runner.py`에 통합했다. root 담당 파일 범위에 이 runner를 추가한다. 원 전체 workspace proof와 현재 source·기본 실행파일 SHA를 확인한 뒤 immutable 복사, 두 C# 컴파일, 단일 native 실행, 원 source/compiled SHA 전후 대조를 수행한다. 수집 성공도 REVIEW_REQUIRED로 남기며 실제 PNG와 동일 Job/Attempt의 DB 대조를 별도로 요구한다. runner는 아직 native 실행하지 않았다.

최초 worker C# 컴파일은 통과했지만 observer는 Program 내부 Require helper 누락으로 CS0103 컴파일 실패였다. root가 helper를 추가했으며 원 실패와 자연 Job 정리·input desktop 보존을 기록했다. 앱 숨김·복귀 실패로 해석하지 않고 fresh evidence에서 다시 실행한다. 두 runner의 Python 구문 검사는 통과했다.

두 C# 컴파일은 통과했으나 최초 native 실행은 관측기가 yt-dlp 입력란을 빈 값으로 가정해 실패했다. 제품은 DLPAD_YTDLP로 원 합성 worker 경로를 미리 채우며 source에서 이 동작을 확인했다. root가 중복 입력 대신 UIA read 값과 원 pin 경로의 정확한 일치를 검사하도록 교정했다. 원 소유 앱 강제 cleanup과 private/input 보존을 기록했고 실제 숨김·복귀는 아직 인수 전이다.

현재 기본 CPU b278ef…에서 두 C# 컴파일 및 실제 active-background 관측은 exit 0이다. 원 app/worker PID·생성 시각·image·Job을 유지한 채 창 숨김 동안 heartbeat 12→18, secondary exit 0 및 원 창 복귀, 같은 worker 유지, Stop 후 원 worker terminal 및 앱 exit 0·자연 Job0·private/input 정리를 확인했다. root가 실제 active/restored PNG를 모두 보았으며 복귀 후 진행률과 중단 컨트롤이 읽히는 것을 확인했다. root의 읽기 전용 DB 대조는 단 하나의 Job/Attempt, 최종 Cancelled·산출물 없음·integrity ok·DB SHA 불변·sentinel 보존을 확인했다. [통합 검토](../validation/evidence/windows-current-cpu-background-prefill-20261006-continue/root-review.json).

숨김 전 DB 사본을 만들지 않았으므로 전체 DB의 전후 논리 보존이라고 주장하지 않는다. 원 worker terminal은 확인했지만 worker exit code를 별도로 읽지 않았다. 수집 완료 이후 root가 독립 스캔 진단용 fixture를 빌드하여 release target 파일은 달라졌으며, 원 수집에 사용한 기본 앱의 immutable 복사 SHA는 유지된다. 제품 source와 현재 기본 빌드 proof는 일치한다. 숨긴 채 완료, 실제 tray·DPI 및 외부 실제 도구는 별도다. 이 작업의 active hide/restore/Stop 범위만 완료하며 전체 P0는 미완료다.
