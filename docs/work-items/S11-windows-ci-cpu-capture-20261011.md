# S11: Windows CI CPU UI 검증 기록 통합

- 상태: in-progress
- 요구사항: R13
- 담당: root
- 수정 파일: scripts/ci_whole_receipt.py, scripts/test_ci_whole_receipt.py, .github/workflows/ci.yml
- 의존성: 기존 CI7 lifecycle·source 계약
- 인수 조건: 기본 renderer를 유지하면서 CPU UI 테스트를 captured clippy/test에 포함; 원argv 거절 회귀 및 새 실제 Windows CI7 검증; Linux/mac 수동 정책 유지.

# Proposed: Windows local CI7 CPU UI feature coverage

범위: Windows ci_capture_operation가 사용하는 shared REQUIRED clippy/test exact argv에 --features dlpad/cpu를 추가. scripts/ci_whole_receipt.py 및 scripts/test_ci_whole_receipt.py와 .github/workflows/ci.yml 후보변경. root통합전 Proposed; schema/status/작업수/다른OS 수동 명령/GUI권한 변경0.

근거: current local CI7 test.stdout 실제39harness603passed/0failed/22ignored. 원 captured clippy/test REQUIRED는 --workspace defaultsoftware이고 cpu required-feature UI binary를 포함하지않음. 전체 app CPU Windows검증1087과동일범위아님. workflow에는별도CPUclippy/test --features dlpad/cpu 단계가이미존재하므로 원격workflow 전체가CPU를전혀시험하지않는다는주장은틀림. 다만CI7 whole artifact/cleanup에그2단계가바인딩되지않고localCI7은실행하지않음.

후보 exact argv:
clippy: cargo clippy --workspace --features dlpad/cpu --all-targets --locked --offline -- -D warnings
test: cargo test --workspace --features dlpad/cpu --locked --offline

원capture는 REQUIRED를import하므로 shareddictionary만변경하면실행과receipt원argv둘다동일. 추가Cargo호출0/7capture유지. 기존defaultsoftware 유지+cpu/alternative/softbuffer/egui/accesskit graph 활성화; 새dependency/lock/schema0. CPUUI관련 컴파일/테스트 시간·메모리·targetdisk 비용이늘지만실제값은새nativeCI7실행후계측필요. root공식MSVChost과freshtarget/lockedcache에서actualCPU회귀포함을raw test 이름/harness집계로확인해야함. 단위Python PASS는native테스트 실행아님. 원workflow의Linux/mac elsecommands defaultflags변경없음. 원격별도CPU lint/test 단계는Windows에서만skip하므로Windows중복2passes는새workflow조건으로skip하고Linux/mac기존추가2passes는유지.

pure인수: 기존16+새2=18 tests PASS(0.094s). default-feature만성공한 clippy/test argv 거절, alternative 잘못feature/locked누락 거절, 기존확정source/log/ID/Job claimfalse guards 유지. 첫테스트후보는테스트에서 REQUIRED list를변경해wrongfeature가허용되는오류1건이났고, argv를copy하여purefixture만변경하도록교정후18PASS. 제품/native RED 주장0.

실제native인수(미실행): updated exactCPU7 stdout/stderr+lifecycles0/naturalJob0/parent/raw/drain, actualtest집계+CPU UI named regression포함; freshMSVChost sourcecommit beforeafter/targetartifact4/IDs/원도구cachesource보존. 새sourceguard/proof필수. 원603receipt/sourceinputs는immutable historical default-feature scope로보존하고 새CPU 검사로승격하거나원receipt argv를수정하지않음. 새validator로옛603 proof 거절은정책확대이며기존데이터변조아님.

기존workflowcapturebranches는명령 --name clippy/test이그대로. 별도CPU candidate lint/test에 if runner.os != Windows 조건을추가하여Windows중복2passes제거(새Cargo호출0). 필요문서에 Windows captures now enable cpu / separateCPUsteps redundant라는정확설명만root추가. remoteExecution/GUI/legal/releasefalse 그대로.

root가 patch dry-run 및 세 파일 통합을 수행했다. python3 -m unittest discover -s scripts -p test_ci_*.py: 68개 PASS. 새 실제 CPU-feature CI7은 후속이며 과거603개 native 기록을 현재 기능 범위로 재사용하지 않는다.

최신 CPU-feature export 실제1,812files를freshownedsource/runtime에준비했다. 첫실행은export에서제외한evidence 부모디렉터리부재로Cargo전거절됐다. 빈부모만create-only하여freshouter출력두번째실행했으나 과거806bf release보존핀을계승해현재whole-fourth2f45 CPU와불일치함을검출했다. 두원failure/log/Job0를보존하고검증guard는완화하지않는다. 세번째freshconfig에서명시적으로현재whole 원release3 핀을기준으로새작업baseline을바인딩하며oldconfig/oldsource/원Rusthost/lock핀은보존한다. 실제CI7명령실행은아직0개이며PASS미주장.

새Windows CPU capture계약은sourceexport에포함했으나 native전체실행준비실패를해결중이다. UI기능동시검증에서 발견한제품worker/Show문제와 CI환경baseline문제는독립작업으로관리한다. 원결과는실패로유지하고추후업데이트source·전체검증·Gitbundle을동일최종snapshot으로결합한다.

실제 frozen source1812의Windows local CI7 third는7명령모두자연exit0/Job0, test1087/0/28/43harness로종료했고root의원raw/1812파일/tool10/preservation6/4artifact/sourcecommit/before-after독립검토PASS_LOCAL_NATIVE_CI7_RECEIPTS_ONLY를통과했다. 이후Shell초기힌트수정이추가되어이결과는frozen1812범위다. 최신source/whole-fifth용fresh입력준비후실제CI7최종실행한다. hosted Actions/auth/원격업로드/정식릴리스완료아님.
