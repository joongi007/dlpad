# S04: 공식 Windows FFmpeg ZIP 관리 설치 인수

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R11
- 담당/수정 범위: root — scripts/windows_ffmpeg_managed_install_probe.py 및 새 evidence
- 선행 작업: v2 mapped ZIP reader, 공식 원ZIP/component SHA 및 nativeversion

## 사용자 결과

원공식ZIP을기존install_managed_archive example으로실제Windows에서비활성설치/삭제하고별도임시root에서정확version확인선택/active삭제거절을검증한다.

## 계약과 인수

private nativeCLI host/소유Job/temporarymanager/고정manifest/원ZIP전후SHA를사용한다. WindowsGNU실제build 및 local/local-verified 세명령sourcebeforeafter/privateinputdesktop일치를검증한다. license문구는공급채택미완료로표시한다. 외부media/Documents/계정은사용하지않으며구현전계획을PASS로표시하지않는다.

## 검증과 handoff

reader통합READY뒤실행한다. 실제GUI/자동설치공급신뢰/설치crash/자동회수는후속이다.

### 첫 실제 Windows 인수

windows-ffmpeg-mapped-zip-install-first build/local/local-verified 전부exit0/PASS. root가각원로그JSON과원archive현재SHA를독립검토하여root-review.json에기록했다. 비활성설치31.128초/삭제검증12.102초, 별도정확version설치30.561초/활성선택검증10.012초, active삭제거절/원ZIP보존/privateinputdesktop유지를확인했다. peakRSS미측정이며대소문자ancestor보강뒤재검증이필요하다. 공급신뢰/GUI자동설치완료로확대하지않는다.
