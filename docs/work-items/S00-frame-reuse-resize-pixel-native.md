# S00: 재사용 프레임의 실제 resize·expose 화면 인수

- 상태: completed
- 분류: verification
- 우선순위: P0
- 요구사항: R03, R06, R08
- 담당/수정 범위: root scripts/windows_frame_resize_pixel_probe.cs 통합·native실행·증거/문서; soak_recovery_audit /tmp C# 초안만
- 선행 작업: S00-bounded-identical-frame-raster-reuse; 현재strict GUI CPU측정종료

## 사용자 결과

합성1만파일목록이있는원native창의크기를줄였다늘리고다시복구할때실제내용이남거나사라지는오류를검증한다.

## 계약과 범위

기존collector는보존하고새standaloneC#에원PI/HWND/creation/Job/privateDesktop/원binary·source pins를고정한다. SetWindowPos NOACTIVATE|NOZORDER로720x520→1100x800→원client크기를요청하고GetClientRect·실제row/PNG를관측한다. expose요청은render실행증거와구분한다. 사용자Desktop/전역키보드/clipboard/실제영상/시스템DPI변경0. sourcePins≤1MiB/result≤256KiB·90sMTA/120sJob·원fixturebytes/DB/sentinel보존은root대조한다.

## 인수 조건

원hash/기하·UIA와실제파일영역pixel·원Job自然0/privateclosed/inputunchanged. 원compile/native실패보존,단순UIA나원hash차이를pixel정확성전체로승격하지않는다. DPI/scale/작업중hide는별도이다.

## 검증과 handoff

/tmp/dlpad-frame-resize-pixel-probe.cs SHA9474f3518835416ad04a1b3e7051052f7e9a3a3930ca03c5c2e2c6aa67663eee 초안이며미컴파일/미실행. 현재CPU측정동안nativecompile/무거운시험은금지하고종료후root가최소통합한다.

초안 독립 검토 후 root가 `scripts/windows_frame_resize_pixel_probe.cs`로 통합했다. 최초 960×640과 양축 축소/확대, 정확한 크기 복귀, source pin 수명 보존 및 전후 SHA, PNG CreateNew·16MiB 상한, REVIEW_REQUIRED 상태를 추가했다. UIA timeout에는 cooperative Stop과 소유 Job 종료 후 bounded join을 사용하고, 살아 있는 worker가 원 HANDLE을 사용할 수 있으면 닫지 않은 채 실패를 기록한다. UIA 호출의 강제 중단은 보장하지 않으며 외부 host 기한이 필요하다. 통합 SHA c0ea90ae5a674a7223945837cbcff1df048b5bf2432e85cf87bc64500a43e6c2. 네이티브 컴파일·실행·실제 이미지 인수는 아직 하지 않았다. root 담당 runner 파일은 `scripts/windows_frame_resize_pixel_native_probe.py`이며 diagnostic_cache는 /tmp 초안만 작성한다.

root가 runner를 통합하고 DB 전체 10,000행의 정확한 root·파일 경로 집합·크기·present·identity 형태 및 중복 검사를 보강했다. 실제 native Job 종료 후에만 모든 payload를 확인한다. `python3 scripts/test_windows_frame_resize_pixel_native_probe.py`는 확장 Windows 경로, foreign root/중복, 틀린 전체 목록·identity·present, symlink·상한·traversal 실패 경로 4개를 통과했다. 담당 파일에 이 테스트와 `.github/workflows/ci.yml`의 Python 검증 단계를 추가한다. 네이티브 컴파일과 실제 이미지 검증은 여전히 남아 있다.

최초 Windows C# 컴파일은 short-circuit 뒤 Rect가 확실히 초기화되지 않았다는 CS0170 네 건으로 실패했다. 원 실패 evidence를 보존하고 두 Rect를 명시적으로 초기화했다. 이 실패는 앱 렌더 결과가 아니며 새 fresh evidence에서 다시 컴파일·실행한다.

수정 후 C# 컴파일과 최신 기본 CPU b278ef…의 실제 private desktop 시험은 exit 0이다. 720×520 → 1100×800 → 960×640의 정확한 크기·원 창 복귀·원 Job 자연0 및 private/input 정리를 확인했다. runner가 전체 10,000개 3byte payload와 DB의 정확한 10,000행·root/경로·present/size/identity 형태·무결성·sentinel을 대조했다. root는 resize 및 expose PNG 6개를 모두 직접 보았고 한글 파일 행·컨트롤·footer가 각 크기에서 읽히며 빈 화면이나 이전 크기의 잔상을 관측하지 않았다. [화면 범위 검증](../validation/evidence/windows-frame-resize-pixel-fixed-20261006-continue/root-pixel-review.json).

수집 완료 시 입력 pins는 일치했다. 이후 root가 독립적인 CPU 진단 collector 선택을 표준 Python helper에 추가해 현재 helper SHA는 당시 수집 SHA와 다르다. 이 변경 및 범위를 root 검토에 명시했으며 제품 Rust·실제 관측 C#·payload 검사 함수는 변경하지 않았다. 모든 native file ID 재조회, 실제 DPI/IME, 내부 surface 재사용 횟수는 검증하지 않았다. 이 작업의 resize/expose 인수만 완료이며 스캔 시간·숨김·테마·전체 P0 인수는 별도다.
