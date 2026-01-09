#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Pawn.h"
#include "OceanSolver.h"
#include "InputActionValue.h" // Required for Enhanced Input
#include "InteractableInterface.h"
#include "MasterShip.generated.h"

UCLASS()
class PROJABYSS_API AMasterShip : public APawn, public IInteractableInterface
{
    GENERATED_BODY()

public:
    AMasterShip();

protected:
    virtual void BeginPlay() override;
    // When possessed by a controller, ensure input mapping is applied
    virtual void PossessedBy(AController* NewController) override;

public:
    virtual void Tick(float DeltaTime) override;
    virtual void SetupPlayerInputComponent(class UInputComponent* PlayerInputComponent) override;

    // -- COMPONENTS --
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Ship")
    UStaticMeshComponent* HullMesh;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Camera")
    class USpringArmComponent* CameraBoom;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Camera")
    class UCameraComponent* FollowCamera;

    // Pontoons
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Buoyancy")
    USceneComponent* PontoonFL;
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Buoyancy")
    USceneComponent* PontoonFR;
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Buoyancy")
    USceneComponent* PontoonBL;
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Buoyancy")
    USceneComponent* PontoonBR;

    // -- INPUT (New Enhanced Input) --
    
    // The "Keyboard Map"
    UPROPERTY(EditAnywhere, BlueprintReadOnly, Category = "Input")
    class UInputMappingContext* DefaultMappingContext;

    // The "W/S" Action
    UPROPERTY(EditAnywhere, BlueprintReadOnly, Category = "Input")
    class UInputAction* MoveAction;

    // The "A/D" Action
    UPROPERTY(EditAnywhere, BlueprintReadOnly, Category = "Input")
    class UInputAction* TurnAction;

    // -- SETTINGS --
    UPROPERTY(EditAnywhere, Category = "Buoyancy")
    float FloatForce = 40000.0f;

    UPROPERTY(EditAnywhere, Category = "Buoyancy")
    float WaterDrag = 2.0f;

    UPROPERTY(EditAnywhere, Category = "Movement")
    float EnginePower = 500000.0f;

    UPROPERTY(EditAnywhere, Category = "Movement")
    float TurnSpeed = 200000.0f;

    // -- INTERACTION --
    // Override from Interactable Interface (BlueprintNativeEvent requires _Implementation)
    virtual void Interact_Implementation(APawn* InstigatorPawn) override;

private:
    FOceanSolver OceanSolver;
    
    float CurrentThrottle; 
    float CurrentSteering;

    // Updated Functions for Enhanced Input
    void MoveForward(const FInputActionValue& Value);
    void TurnRight(const FInputActionValue& Value);

    // Helper
    void ApplyInputMappingToController(AController* InController);
};