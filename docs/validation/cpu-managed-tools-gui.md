# CPU 관리 도구 GUI harness 준비

신규 scripts/cpu_tools_probe.py는 기존 managed_retained_probe.seed의 합성 두 버전/owner marker/activation.sqlite 계약을 사용한다. 공식 설치 pin과 executable digest를 이 합성 fixture로 검증했다고 표현하지 않는다. inspect/select-remove-use/closewait case를 제공하며 새 출력 폴더·고정 expected binary SHA·명시 font가 필요하다.

사용자 desktop과 분리된 owned Xvfb 및 private DBus/AT-SPI를 만들고 enabled node bounds로 클릭한다. fixture media는 빈 소유 폴더이며 외부 파일은 소유 fixture의 immutable bytes다. select/remove는 read-only active DB와 exact bytes로 검증하고 Use는 도구 목록이 아닌 다운로드 탭에 선택 경로가 나타나는지 확인한다. CloseWait는 activation.sqlite의 owned EXCLUSIVE lock으로 진행 작업을 만든 뒤 WM_DELETE_WINDOW와 중단 후 종료를 요청하고 lock 유지 중 생존·해제 후8초 이내 종료를 관측한다. 모든 GUI/DBus는 소유 process group으로만 정리하고 새 evidence/result/raw/PNG를 보존한다.

install 버튼은 누르지 않으며 신규 네트워크 다운로드나 실제 계정은 없다. 따라서 실제 install cancel을 확인하지 않는다. 별도 실제 공급 설치 취소/timeout은 기존 engine 회귀와 후속 opt-in GUI 인수로 유지한다. 합성 CloseWait도 install network cancel을 대신하지 않는다.

python3 -m py_compile scripts/cpu_tools_probe.py exit0, --validate-only 인자 검사 exit0. GUI 프로세스는 아직 실행하지 않았으며 root READY 전 실행 금지다. 실제 tab/label/AT-SPI tree는 inspect 후 확인해야 하고 harness 작성은 GUI 성공 증거가 아니다. 패널 source/Cargo/기존 harness는 수정하지 않았다.

## 실제 Linux GUI 결과

root READY SHA256 522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd로 inspect, select-remove-use, closewait를 실행했다. 최종 inspect/select-remove-use-textfix/closewait는 passed=true, owned_cleanup=true, binary before/after 동일이다. 합성v2 활성화·보관v1 제거·활성 파일 및 외부bytes 보존·active행 select/remove 비활성·다운로드 입력의 role79 Text.GetText 정확 경로를 확인했다. selected.png를 직접 읽어 active행 버튼 상태와 보관 행을 확인했다.

첫 select-remove-use는 최종 collector assertion이 Properties.Name에서 입력값을 찾는 오류로 failed였다. 실제 raw Text.GetText에는 선택된 경로가 있었다. harness만 Text 인터페이스 exact 비교로 정정하고 fresh textfix prefix를 실행해 통과했다. 첫 실패raw를 유지하고 제품 source는 변경하지 않았다.

CloseWait는 owned EXCLUSIVE activation DB lock 유지 중150ms 앱 생존을 확인했고 lock 해제 후63.7ms에 exit0으로 종료했다. 내부 atomic이나 다운로드 취소를 직접 관측한 증거로 확대하지 않는다. 모든 실행은 privateXvfb/privateDBus이며 신규 공식 다운로드·실제미디어·계정·사용자desktop은 없다. 실제 install cancel 및 Windows/macOS GUI는 미검증이다. 원시 증거는 docs/validation/evidence/cpu-tools-inspect, cpu-tools-select-remove-use(초기실패), cpu-tools-select-remove-use-textfix, cpu-tools-closewait에 보존한다.
